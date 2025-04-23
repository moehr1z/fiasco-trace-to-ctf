mod converter;
mod event;
mod helpers;
mod parser;

use crate::event::Event;
use babeltrace2_sys::LoggingLevel;
use converter::Converter;
use converter::opts::Opts;
use core::str;
use log::warn;
use log::{debug, error, info};
use parser::EventParser;
use std::collections::VecDeque;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::{join, task};

const IP_ADDRESS: &str = "0.0.0.0:8888";

// enum ConvMsg {
//     Meta(Vec<u8>),
//     Stream(Vec<u8>),
// }

#[tokio::main]
async fn main() {
    env_logger::init();

    // network -> parser
    let (net_tx, mut parser_rx) = mpsc::channel(32);
    // parser -> converter
    let (parser_tx, mut converter_rx) = mpsc::channel::<Event>(32);
    // converter -> live session
    // let (converter_tx, mut live_rx) = mpsc::channel(32);

    let event_buf: Arc<Mutex<VecDeque<Event>>> = Arc::new(Mutex::new(VecDeque::new()));

    // TODO error handling
    // TODO name mapping

    // Receive the event bytes from the network and pass them to the parser
    let network_handle = task::spawn(async move {
        info!("Listening on {}", IP_ADDRESS);
        let listener = TcpListener::bind(IP_ADDRESS).await.unwrap();
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Accepted connection from {:?}", addr);
                let mut reader = BufReader::new(stream);
                let mut buf: [u8; 128] = [0; 128];

                while reader.read_exact(&mut buf).await.is_ok() {
                    net_tx.send(buf).await.unwrap();
                    debug!("Read and sent event bytes");
                }
            }
            Err(e) => error!("Error accepting TCP connection ({:?})", e),
        }
    });

    // Parse the event bytes and pass the to the converter
    let event_buf_c = event_buf.clone();
    let parser_handle = task::spawn(async move {
        let mut first_event_observed = false;
        let mut biggest_event_num: u64 = 0;

        while let Some(event_bytes) = parser_rx.recv().await {
            debug!("Received event bytes");
            let mut reader = Cursor::new(event_bytes);
            let event = EventParser::next_event(&mut reader);

            match event {
                Ok(event) => {
                    if let Some(e) = event {
                        let event_number = e.event_common().number;
                        debug!("Event count: {event_number}");
                        if event_number > biggest_event_num || !first_event_observed {
                            let dropped_events = if first_event_observed {
                                event_number - biggest_event_num - 1
                            } else {
                                0
                            };
                            if dropped_events > 0 {
                                warn!("Dropped {dropped_events} events");
                            }
                            biggest_event_num = event_number;
                            first_event_observed = true;
                            parser_tx.send(e).await.unwrap();
                            debug!("Parsed and sent event");
                        } else {
                            // TODO this should somehow be handled instead of just dropping it, but
                            // i believe the best way is to prevent it in the first place with a
                            // buffer redesign
                            debug!(
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
    });

    // Convert the events to CTF and pass the to the disk writer and live streamer
    let local = task::LocalSet::new();
    local
        .run_until(async move {
            let ctf_dir_path = "/dev/shm/ctf_trace/";
            let ctf_dir = Path::new(ctf_dir_path); // we use a tmpfs dir
            // because babeltrace only has a file system ctf sink, but we don't want to read the
            // data in again from disk to send it to the live session
            let opts = Opts {
                clock_name: "monotonic".to_string(),
                trace_name: "l4re".to_string(),
                log_level: LoggingLevel::Warn,
                output: ctf_dir.to_str().unwrap().into(),
            };
            let eof_signal = Arc::new(AtomicBool::new(false));
            let mut conv = Converter::new(event_buf.clone(), eof_signal.clone(), opts).unwrap();

            // let mut stream_file_path = ctf_dir_path.to_string();
            // stream_file_path.push_str("stream");
            // println!("PATH: {}", stream_file_path);
            // let mut stream_file = File::create(&stream_file_path).await.unwrap();
            // let mut stream_buf = Vec::new();

            // let mut meta_file_path = ctf_dir_path.to_string();
            // meta_file_path.push_str("metadata");
            // let mut meta_file = File::open(&meta_file_path).await.unwrap();
            // let mut meta_buf = Vec::new();

            let _ = task::spawn_local(async move {
                while let Some(event) = converter_rx.recv().await {
                    debug!("Received event");
                    {
                        let mut event_buf = event_buf_c.lock().unwrap();
                        event_buf.push_back(event);
                    }
                    debug!("Trying to convert event...");
                    match conv.convert_once() {
                        Ok(_) => {
                            debug!("Succesfully converted event");

                            // stream_file.read_to_end(&mut stream_buf).await.unwrap();

                            // send to live session handler
                            // TODO don't copy
                            // converter_tx
                            //     .send(ConvMsg::Stream(stream_buf.clone()))
                            //     .await
                            //     .unwrap();

                            // TODO commit to disk
                        }
                        Err(e) => error!("Error converting event ({:?})", e),
                    }
                }

                debug!("Closing converter stream...");
                eof_signal.store(true, Relaxed);
                match conv.convert() {
                    Ok(_) => debug!("Succesfully closed converter stream"),
                    Err(e) => error!("Error closing converter stream ({:?})", e),
                }
            })
            .await;
        })
        .await;

    let _ = join!(network_handle, parser_handle);

    //
    // let converter = task::spawn_blocking(move || {
    //     let mut converter = Converter::new(event_buf, HashMap::new()).unwrap();
    //     converter.convert().unwrap();
    // });

    // Sort
    // events.sort_by_key(|e| e.event_common().tsc);

    // create mapping DB of pointer -> name, timestamp until valid
    // let mut name_db: HashMap<L4Addr, Vec<(String, Option<u64>)>> = HashMap::new(); // vector of entries because you could reassign a pointer or rename the object
    // for e in &events {
    //     let ts = e.event_common().tsc;
    //
    //     // TODO are these all relevant events?
    //     match e {
    //         Event::Nam(ev) => {
    //             let addr = ev.obj;
    //             let addr = addr & 0xFFFFFFFFFFFFF000; // TODO somehow the last 3 bits of ctx are always 0, look up why
    //
    //             let entry = name_db.get_mut(&addr);
    //
    //             let bind = &ev.name.iter().map(|&c| c as u8).collect::<Vec<u8>>();
    //             let name = str::from_utf8(bind).unwrap(); // TODO error handling
    //             let name = name.replace('\0', "");
    //             if name.is_empty() {
    //                 continue;
    //             }
    //
    //             match entry {
    //                 None => {
    //                     name_db.insert(addr, vec![(name, None)]);
    //                 }
    //                 Some(entry) => {
    //                     // there already were some names for this pointer
    //                     for e in &mut *entry {
    //                         // the pointer was renamed so the prev name is only valid until the time of rename
    //                         if e.1.is_none() {
    //                             e.1 = Some(ts);
    //                         }
    //                     }
    //                     entry.push((name, None));
    //                 }
    //             }
    //         }
    //         Event::Destroy(ev) => {
    //             let addr = ev.obj;
    //             let entry = name_db.get_mut(&addr);
    //
    //             // if it is none, some unnamed object was destroyed, so we don't care
    //             if let Some(entry) = entry {
    //                 for e in &mut *entry {
    //                     // deleting the objects invalidates its name at time of deletion
    //                     if e.1.is_none() {
    //                         e.1 = Some(ts);
    //                     }
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}
