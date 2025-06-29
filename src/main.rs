mod converter;
mod event;
mod helpers;
mod opts;
mod parser;

use crate::converter::interruptor::Interruptor;
use crate::event::Event;
use clap::Parser;
use converter::Converter;
use core::str;
use log::warn;
use log::{debug, error, info};
use opts::Opts;
use parser::EventParser;
use regex::Regex;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::io::{self, BufReader, Cursor, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Instant;
use std::{fs, thread};

const IP_ADDRESS: &str = "0.0.0.0:8888";

fn main() {
    let opts = Opts::parse();
    let opts_c = opts.clone();

    env_logger::init();

    let intr = Interruptor::new();
    let intr_clone = intr.clone();
    ctrlc::set_handler(move || {
        if intr_clone.is_set() {
            let exit_code = if cfg!(target_family = "unix") {
                // 128 (fatal error signal "n") + 2 (control-c is fatal error signal 2)
                130
            } else {
                // Windows code 3221225786
                // -1073741510 == C000013A
                -1073741510
            };
            std::process::exit(exit_code);
        }

        debug!("Shutdown signal received");
        intr_clone.set();
    })
    .unwrap();

    // network -> parser
    let (net_tx, parser_rx) = mpsc::channel();
    // parser -> converter
    let (parser_tx, converter_rx) = mpsc::channel::<Event>();
    // converter -> live session
    // let (converter_tx, mut live_rx) = mpsc::channel(32);

    // TODO error handling

    // Receive the event bytes from the network and pass them to the parser
    let network_handle = thread::spawn(move || {
        info!("Listening on {}", IP_ADDRESS);
        let listener = TcpListener::bind(IP_ADDRESS).unwrap_or_else(|_| {
            error!("Could not bind to provided address/port!");
            panic!();
        });
        let mut events_received: u64 = 0;
        let mut start_time: Option<Instant> = None;

        match listener.accept() {
            Ok((stream, addr)) => {
                println!("Accepted connection from {:?}", addr);
                let mut reader = BufReader::new(stream);
                let mut buf: [u8; 128] = [0; 128];

                while reader.read_exact(&mut buf).is_ok() {
                    if start_time.is_none() {
                        start_time = Some(Instant::now());
                    }
                    events_received += 1;
                    match net_tx.send(buf) {
                        Ok(_) => debug!("Parsed and sent event"),
                        Err(e) => {
                            error!("Could not send event to parser ({:?}). Dropping it...", e)
                        }
                    }
                }
            }
            Err(e) => error!("Error accepting TCP connection ({:?})", e),
        }

        let runtime = start_time.unwrap().elapsed();
        let throughput = ((events_received * 128) as f64) / runtime.as_secs_f64();
        throughput
    });

    // Parse the event bytes and pass the to the converter
    let parser_handle = thread::spawn(move || {
        let mut first_event_observed = false;
        let mut biggest_event_num: u64 = 0;
        let mut last_event_tsc: u64 = 0;
        let mut start_time: Option<Instant> = None;
        let mut dropped_events_total: u64 = 0;
        let mut weird_events: u64 = 0;

        while let Ok(event_bytes) = parser_rx.recv() {
            if start_time.is_none() {
                start_time = Some(Instant::now());
            }

            debug!("Received event bytes");
            let mut reader = Cursor::new(event_bytes);
            let event = EventParser::next_event(&mut reader);

            match event {
                Ok(event) => {
                    if let Some(e) = event {
                        let event_number = e.event_common().number;
                        let event_tsc = e.event_common().tsc;
                        debug!("Event count: {event_number}");
                        if event_number > biggest_event_num || !first_event_observed {
                            let dropped_events = if first_event_observed {
                                event_number - biggest_event_num - 1
                            } else {
                                0
                            };
                            if dropped_events > 0 {
                                dropped_events_total += dropped_events;
                                warn!(
                                    "Dropped {dropped_events} events (event num: {event_number}, biggest event num: {biggest_event_num}"
                                );
                            }

                            if event_tsc < last_event_tsc {
                                warn!("Rising event number, but falling timestamp! \n
                                    event number: {event_number} \t biggest event number: {biggest_event_num} \n
                                    event tsc: {event_tsc} \t last event tsc: {last_event_tsc}");
                                weird_events += 1;
                                continue;
                            }

                            biggest_event_num = event_number;
                            last_event_tsc = event_tsc;
                            first_event_observed = true;
                            match parser_tx.send(e) {
                                Ok(_) => debug!("Parsed and sent event"),
                                Err(e) => error!(
                                    "Could not send event to converter ({:?}). Dropping it...",
                                    e
                                ),
                            }
                        } else {
                            warn!(
                                "Found duplicate/out of order event (event nr: {event_number}, max nr: {biggest_event_num}"
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!("Could not parse event ({:?})", e);
                }
            }
        }

        println!("THERE WERE {weird_events} WEIRD EVENTS");

        (start_time, dropped_events_total)
    });

    // Convert the events to CTF and pass the to the disk writer and live streamer
    let converter_handle = thread::spawn(move || {
        // because babeltrace only has a file system ctf sink, but we don't want to read the
        // data in again from disk to send it to the live session
        let eof_signal: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let mut converters: HashMap<u8, Converter> = HashMap::new();
        // TODO put the event stream in the converter and make a func to return it
        let mut event_streams: HashMap<u8, Rc<RefCell<VecDeque<Event>>>> = HashMap::new();
        // TODO make own struct for the name map
        let name_map: Rc<RefCell<HashMap<u64, (String, String)>>> =
            Rc::new(RefCell::new(HashMap::new())); // ctx pointer -> (name, dbg_id)
        let mut nr_conv_events: u64 = 0;

        while let Ok(event) = converter_rx.recv() {
            let cpu_id = event.event_common().cpu;
            converters.entry(cpu_id).or_insert_with(|| {
                let event_buf: Rc<RefCell<VecDeque<Event>>> =
                    Rc::new(RefCell::new(VecDeque::new()));
                event_streams.insert(cpu_id, event_buf.clone());

                let mut opts_c = opts.clone();
                opts_c.output = format!("{}_{cpu_id}", opts_c.output.to_str().unwrap()).into(); // TODO unwrap
                debug!("Instantiating converter {cpu_id}");
                Converter::new(
                    event_buf,
                    eof_signal.clone(),
                    opts_c,
                    cpu_id,
                    intr.clone(),
                    name_map.clone(),
                )
                .unwrap_or_else(|_| {
                    error!("Could not instantiate converter!");
                    panic!();
                })
            });

            let conv = converters.get_mut(&cpu_id).unwrap();

            debug!("Received event \n {:?}", event);
            {
                event_streams
                    .get_mut(&cpu_id)
                    .unwrap()
                    .borrow_mut()
                    .push_back(event);
            }
            debug!("Trying to convert event...");
            match conv.convert_once() {
                Ok(_) => {
                    debug!("Succesfully converted event");
                    nr_conv_events += 1;

                    // TODO send to live session handler
                    // TODO commit to disk
                }
                Err(e) => error!("Error converting event ({:?})", e),
            }
        }

        debug!("Closing converter stream...");
        eof_signal.set(true);
        for conv in converters.values_mut() {
            match conv.convert() {
                Ok(_) => debug!("Succesfully closed converter stream"),
                Err(e) => error!("Error closing converter stream ({:?})", e),
            }
        }

        // retrun the cpus of which we saw events, so we can merge those streams later
        (converters.into_keys().collect::<Vec<u8>>(), nr_conv_events)
    });

    let rcv_throughput = network_handle.join().unwrap();
    let (start_time, dropped_events) = parser_handle.join().unwrap();
    let (cpus, conv_events) = converter_handle.join().unwrap();

    println!("EVENTS TOTAL: {conv_events}");
    if let Some(start) = start_time {
        let runtime = start.elapsed();
        let throughput = (conv_events as f64) / runtime.as_secs_f64();
        println!("THROUGHPUT: {throughput} (EVENTS/SEC)");
    } else {
        error!("Start time is None!");
    }
    println!("EVENTS DROPPED: {dropped_events}");
    println!("RECEIVE THROUHGPUT: {rcv_throughput}");
    println!("NR CPUS: {}", cpus.len());

    merge_traces(cpus, opts_c.output).unwrap();
}

fn merge_traces(cpus: Vec<u8>, path: PathBuf) -> Result<(), io::Error> {
    // Define input directories and output
    let out_dir = Path::new(&path);

    // Create output directory if it doesn't exist
    if !out_dir.exists() {
        fs::create_dir(out_dir)?;
    }

    // Move and rename stream files
    for cpu in &cpus {
        let src_dir = PathBuf::from(format!("{}_{}", path.display(), cpu));
        let stream_file = src_dir.join("stream");
        let dest_file = out_dir.join(format!("stream_{}", cpu));
        fs::copy(&stream_file, &dest_file)?;
    }

    // Merge metadata files
    let mut merged = String::new();
    let mut seen_sections = Vec::new();
    let section_re = Regex::new(r"^(trace|env|clock) \{").unwrap();
    let mut include_lines = true;

    for cpu in cpus {
        let meta_path = PathBuf::from(format!("{}_{}/metadata", path.display(), cpu));
        let mut file = fs::File::open(&meta_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        for line in content.lines() {
            if let Some(cap) = section_re.captures(line) {
                let sec = cap.get(1).unwrap().as_str();
                if seen_sections.contains(&sec.to_string()) {
                    // Skip lines until matching closing brace
                    include_lines = false;
                } else {
                    seen_sections.push(sec.to_string());
                    include_lines = true;
                }
            }

            if include_lines {
                merged.push_str(line);
                merged.push('\n');
            }

            // Detect end of section
            if !include_lines && line.trim() == "};" {
                include_lines = true;
            }
        }
    }

    // Write merged metadata
    let mut out_meta = fs::File::create(out_dir.join("metadata"))?;
    out_meta.write_all(merged.as_bytes())?;

    debug!("Merged CTF streams into {:?}", out_dir);
    Ok(())
}
