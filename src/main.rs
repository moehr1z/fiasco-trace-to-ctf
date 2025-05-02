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
            })
            .await;
        })
        .await;

    let _ = join!(network_handle, parser_handle);
}
