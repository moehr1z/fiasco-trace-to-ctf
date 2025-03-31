mod converter;
mod parser;

use converter::converter::Converter;
use core::str;
use parser::event::{Event, typedefs::L4Addr};
use parser::parser::EventParser;
use std::collections::VecDeque;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8888").unwrap();
    let mut events_bytes: Vec<u8> = Vec::new();
    let mut events: VecDeque<Event> = VecDeque::new();

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
            events.push_back(e)
        } else {
            break;
        }
    }

    // Sort
    // events.sort_by_key(|e| e.event_common().tsc);

    // create mapping DB of pointer -> name, timestamp until valid
    let mut name_db: HashMap<L4Addr, Vec<(String, Option<u64>)>> = HashMap::new(); // vector of entries because you could reassign a pointer or rename the object
    for e in &events {
        let ts = e.event_common().tsc;

        // TODO are these all relevant events?
        match e {
            Event::Nam(ev) => {
                let addr = ev.obj;
                let addr = addr & 0xFFFFFFFFFFFFF000; // TODO somehow the last 3 bits of ctx are always 0, look up why

                let entry = name_db.get_mut(&addr);

                let bind = &ev.name.iter().map(|&c| c as u8).collect::<Vec<u8>>();
                let name = str::from_utf8(bind).unwrap(); // TODO error handling
                let name = name.replace('\0', "");
                if name.is_empty() {
                    continue;
                }

                match entry {
                    None => {
                        name_db.insert(addr, vec![(name, None)]);
                    }
                    Some(entry) => {
                        // there already were some names for this pointer
                        for e in &mut *entry {
                            // the pointer was renamed so the prev name is only valid until the time of rename
                            if e.1.is_none() {
                                e.1 = Some(ts);
                            }
                        }
                        entry.push((name, None));
                    }
                }
            }
            Event::Destroy(ev) => {
                let addr = ev.obj;
                let entry = name_db.get_mut(&addr);

                // if it is none, some unnamed object was destroyed, so we don't care
                if let Some(entry) = entry {
                    for e in &mut *entry {
                        // deleting the objects invalidates its name at time of deletion
                        if e.1.is_none() {
                            e.1 = Some(ts);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    println!("{:?}", name_db);

    // Convert to CTF
    let mut converter = Converter::new(events, name_db).unwrap();
    converter.convert().unwrap();
}
