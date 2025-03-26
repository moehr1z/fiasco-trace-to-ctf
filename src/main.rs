use l4re_traceconv::converter::Converter;
use l4re_traceparse::event::Event;
use l4re_traceparse::parser::EventParser;
use std::{
    io::{Cursor, Read},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8888").unwrap();
    let mut events_bytes: Vec<u8> = Vec::new();
    let mut events: Vec<Event> = Vec::new();

    match listener.accept() {
        Ok((mut stream, _addr)) => {
            let mut buf: [u8; 128] = [0; 128];

            while stream.read(&mut buf).unwrap() != 0 {
                events_bytes.append(&mut buf.to_vec());
            }
        }
        Err(e) => println!("Error accepting TCP connection ({:?})", e),
    }

    // Parse
    let mut reader = Cursor::new(events_bytes);
    loop {
        let event = EventParser::next_event(&mut reader).unwrap();
        if let Some(e) = event {
            events.push(e)
        } else {
            break;
        }
    }

    // Sort
    events.sort_by_key(|e| e.event_common().tsc);

    // TODO in the converter keep a mapping db

    // Convert to CTF
    let mut converter = Converter::new(events).unwrap();
    converter.convert().unwrap();
}
