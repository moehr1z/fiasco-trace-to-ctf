pub mod error;

const EVENT_SIZE: usize = 128;
const EVENT_TYPE_POSITION: i64 = 44;

use crate::event::Event;
use crate::event::bp::BpEvent;
use crate::event::context_switch::ContextSwitchEvent;
use crate::event::destroy::DestroyEvent;
use crate::event::drq::DrqEvent;
use crate::event::empty::EmptyEvent;
use crate::event::event_type::EventType;
use crate::event::exregs::ExregsEvent;
use crate::event::factory::FactoryEvent;
use crate::event::gate::GateEvent;
use crate::event::ieh::IehEvent;
use crate::event::ipc_res::IpcResEvent;
use crate::event::ipfh::IpfhEvent;
use crate::event::irq::IrqEvent;
use crate::event::migration::MigrationEvent;
use crate::event::nam::NamEvent;
use crate::event::rcu::RcuEvent;
use crate::event::sched::SchedEvent;
use crate::event::svm::SvmEvent;
use crate::event::timer::TimerEvent;
use crate::event::tmap::TmapEvent;
use crate::event::trap::TrapEvent;
use crate::event::vcpu::VcpuEvent;
use crate::event::{ipc::IpcEvent, pf::PfEvent};
use crate::parser::error::Error;
use binrw::BinRead;
use byteorder::ReadBytesExt;
use log::warn;
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

        match event_type {
            EventType::Breakpoint => {
                let event = BpEvent::read(&mut reader)?;
                Ok(Some(Event::Bp(event)))
            }
            EventType::KobjectNames => {
                let event = NamEvent::read(&mut reader)?;
                Ok(Some(Event::Nam(event)))
            }
            EventType::KobjectCreate => {
                let event = FactoryEvent::read(&mut reader)?;
                Ok(Some(Event::Factory(event)))
            }
            EventType::KobjectDestroy
            | EventType::FactoryDelete
            | EventType::KobjectDelete
            | EventType::KobjectDeleteGeneric => {
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
            EventType::DrqHandling => {
                let event = DrqEvent::read(&mut reader)?;
                Ok(Some(Event::Drq(event)))
            }
            EventType::ExRegs => {
                let event = ExregsEvent::read(&mut reader)?;
                Ok(Some(Event::Exregs(event)))
            }
            EventType::IpcGateInvoke => {
                let event = GateEvent::read(&mut reader)?;
                Ok(Some(Event::Gate(event)))
            }
            EventType::IrqObjectTriggers => {
                let event = IrqEvent::read(&mut reader)?;
                Ok(Some(Event::Irq(event)))
            }
            EventType::PageFaultInvalidPager => {
                let event = IpfhEvent::read(&mut reader)?;
                Ok(Some(Event::Ipfh(event)))
            }
            EventType::RcuCall | EventType::RcuCallbacks | EventType::RcuIdle => {
                let event = RcuEvent::read(&mut reader)?;
                Ok(Some(Event::Rcu(event)))
            }
            EventType::SchedulingContextLoad | EventType::SchedulingContextSave => {
                let event = SchedEvent::read(&mut reader)?;
                Ok(Some(Event::Sched(event)))
            }
            EventType::ThreadMigration => {
                let event = MigrationEvent::read(&mut reader)?;
                Ok(Some(Event::Migration(event)))
            }
            EventType::TimerIrqsKernelScheduling => {
                let event = TimerEvent::read(&mut reader)?;
                Ok(Some(Event::Timer(event)))
            }
            EventType::VcpuEvents => {
                let event = VcpuEvent::read(&mut reader)?;
                Ok(Some(Event::Vcpu(event)))
            }
            EventType::VmSvm => {
                let event = SvmEvent::read(&mut reader)?;
                Ok(Some(Event::Svm(event)))
            }
            EventType::ExceptionInvalidHandler => {
                let event = IehEvent::read(&mut reader)?;
                Ok(Some(Event::Ieh(event)))
            }
            EventType::Exceptions => {
                let event = TrapEvent::read(&mut reader)?;
                Ok(Some(Event::Trap(event)))
            }
            EventType::TaskMap | EventType::TaskUnmap => {
                let event = TmapEvent::read(&mut reader)?;
                Ok(Some(Event::Tmap(event)))
            }
            EventType::IpcTrace => {
                warn!("IpcTrace event unknown.");
                Err(error::Error::EventType(event_type_num))
            }
            EventType::KeReg | EventType::KeBin | EventType::Ke => {
                // TODO this is just a placeholder for testing, the real event has yet to be implemented
                warn!("Ke events are not yet properly implemented.");
                let event = EmptyEvent::read(&mut reader)?;
                Ok(Some(Event::Empty(event)))
            }
            EventType::Hidden => {
                let event = EmptyEvent::read(&mut reader)?;
                warn!("Got \"Hidden\" Event \n {:?}", event);
                Err(error::Error::EventType(event_type_num))
            }
            EventType::Unused => {
                let event = EmptyEvent::read(&mut reader)?;
                warn!("Got \"Unused\" Event \n {:?}", event);
                Err(error::Error::EventType(event_type_num))
            }
        }
    }
}
