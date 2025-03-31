pub mod error;
pub mod event;

const EVENT_SIZE: usize = 128;
const EVENT_TYPE_POSITION: i64 = 44;

use crate::parser::error::Error;
use crate::parser::event::Event;
use crate::parser::event::context_switch::ContextSwitchEvent;
use crate::parser::event::destroy::DestroyEvent;
use crate::parser::event::event_type::EventType;
use crate::parser::event::factory::FactoryEvent;
use crate::parser::event::ipc_res::IpcResEvent;
use crate::parser::event::nam::NamEvent;
use crate::parser::event::{ipc::IpcEvent, pf::PfEvent};
use binrw::BinRead;
use byteorder::ReadBytesExt;
use std::io::{ErrorKind, Read, Seek};

pub struct EventParser {}

impl EventParser {
    pub fn next_event<R: Read>(reader: &mut R) -> Result<Option<Event>, Error> {
        let mut buffer: [u8; EVENT_SIZE] = [0; EVENT_SIZE];
        let res = reader.read_exact(&mut buffer);
        match res {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(Error::Io(e)),
        }
        let mut reader = std::io::Cursor::new(&mut buffer);

        reader.seek_relative(EVENT_TYPE_POSITION)?;
        let event_type_num = reader.read_u8()?;
        let event_type: EventType = event_type_num.try_into()?;
        reader.seek_relative(-EVENT_TYPE_POSITION - 1)?;

        // TODO use a macro for this
        match event_type {
            EventType::KobjectNames => {
                let event = NamEvent::read(&mut reader)?;
                Ok(Some(Event::Nam(event)))
            }
            EventType::KobjectCreate => {
                let event = FactoryEvent::read(&mut reader)?;
                Ok(Some(Event::Factory(event)))
            }
            EventType::KobjectDelete => {
                let event = DestroyEvent::read(&mut reader)?;
                Ok(Some(Event::Destroy(event)))
            }
            EventType::KobjectDestroy => {
                let event = DestroyEvent::read(&mut reader)?;
                Ok(Some(Event::Destroy(event)))
            }
            EventType::ContextSwitch => {
                let event = ContextSwitchEvent::read(&mut reader)?;
                Ok(Some(Event::ContextSwitch(event)))
            }
            EventType::Pf => {
                let event = PfEvent::read(&mut reader)?;
                Ok(Some(Event::Pf(event)))
            }
            EventType::Ipc => {
                let event = IpcEvent::read(&mut reader)?;
                Ok(Some(Event::Ipc(event)))
            }
            EventType::IpcRes => {
                let event = IpcResEvent::read(&mut reader)?;
                Ok(Some(Event::IpcRes(event)))
            }
            _ => todo!("Event type not yet implemented ({:?})", event_type_num),
        }
    }
}
