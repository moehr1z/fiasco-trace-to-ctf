mod converter;
mod event;
mod helpers;
mod opts;
mod parser;

use crate::event::Event;
use clap::Parser;
use converter::Converter;
use core::str;
use log::warn;
use log::{debug, error, info};
use opts::Opts;
use parser::EventParser;
use std::collections::VecDeque;
use std::io::{BufReader, Cursor, Read};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;
use std::thread;

const IP_ADDRESS: &str = "0.0.0.0:8888";

fn main() {
    let opts = Opts::parse();

    env_logger::init();

    // network -> parser
    let (net_tx, parser_rx) = mpsc::channel();
    // parser -> converter
    let (parser_tx, converter_rx) = mpsc::channel::<Event>();
    // converter -> live session
    // let (converter_tx, mut live_rx) = mpsc::channel(32);

    let event_buf: Arc<Mutex<VecDeque<Event>>> = Arc::new(Mutex::new(VecDeque::new()));

    // TODO error handling

    // Receive the event bytes from the network and pass them to the parser
    let network_handle = thread::spawn(move || {
        info!("Listening on {}", IP_ADDRESS);
        let listener = TcpListener::bind(IP_ADDRESS).unwrap_or_else(|_| {
            error!("Could not bind to provided address/port!");
            panic!();
        });
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("Accepted connection from {:?}", addr);
                let mut reader = BufReader::new(stream);
                let mut buf: [u8; 128] = [0; 128];

                while reader.read_exact(&mut buf).is_ok() {
                    if net_tx.send(buf).is_ok() {
                        debug!("Read and sent event bytes");
                    } else {
                        warn!("Could not send event to parser. Dropping it...");
                    }
                }
            }
            Err(e) => error!("Error accepting TCP connection ({:?})", e),
        }
    });

    // Parse the event bytes and pass the to the converter
    let event_buf_c = event_buf.clone();
    let parser_handle = thread::spawn(move || {
        let mut first_event_observed = false;
        let mut biggest_event_num: u64 = 0;

        while let Ok(event_bytes) = parser_rx.recv() {
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
                            if parser_tx.send(e).is_ok() {
                                debug!("Parsed and sent event");
                            } else {
                                warn!("Could not send event to converter. Dropping it...");
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
    });

    // Convert the events to CTF and pass the to the disk writer and live streamer
    let converter_handle = thread::spawn(move || {
        // because babeltrace only has a file system ctf sink, but we don't want to read the
        // data in again from disk to send it to the live session
        let eof_signal = Arc::new(AtomicBool::new(false));
        let mut conv =
            Converter::new(event_buf.clone(), eof_signal.clone(), opts).unwrap_or_else(|_| {
                error!("Could not instantiate converter!");
                panic!();
            });

        while let Ok(event) = converter_rx.recv() {
            debug!("Received event");
            {
                let mut event_buf = event_buf_c.lock().unwrap_or_else(|_| {
                    error!("Poisoned lock!");
                    panic!()
                });
                event_buf.push_back(event);
            }
            debug!("Trying to convert event...");
            match conv.convert_once() {
                Ok(_) => {
                    debug!("Succesfully converted event");

                    // TODO send to live session handler
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
    });

    network_handle.join().unwrap();
    parser_handle.join().unwrap();
    converter_handle.join().unwrap();
}
