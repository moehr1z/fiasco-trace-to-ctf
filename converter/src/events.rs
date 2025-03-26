use crate::types::StringCache;
use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;
use enum_iterator::Sequence;
use l4re_traceparse::event::EventType;
use l4re_traceparse::event::{
    context_switch::ContextSwitchEvent, destroy::DestroyEvent, factory::FactoryEvent,
    nam::NamEvent, pf::PfEvent,
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::str;

// TODO - any way to use serde-reflection to synthesize these?

#[derive(CtfEventClass)]
#[event_name = "FACTORY"]
pub struct Factory {
    pub op: i64,
    pub buffer: u64,
    pub id: u64,
    pub ram: u64,
    pub newo: u64,
}

impl TryFrom<FactoryEvent> for Factory {
    type Error = Error;

    fn try_from(value: FactoryEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            op: value.op,
            buffer: value.buffer,
            id: value.id,
            ram: value.ram,
            newo: value.newo,
        })
    }
}

#[derive(CtfEventClass)]
#[event_name = "DESTROY"]
pub struct Destroy {
    pub obj: u64,
    pub id: u64,
    pub type_: u64,
    pub ram: u64,
}

impl TryFrom<DestroyEvent> for Destroy {
    type Error = Error;

    fn try_from(value: DestroyEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            obj: value.obj,
            id: value.id,
            type_: value.type_,
            ram: value.ram,
        })
    }
}

#[derive(CtfEventClass)]
#[event_name = "NAM"]
pub struct Nam<'a> {
    pub obj: u64,
    pub id: u64,
    pub name: &'a CStr,
}

impl<'a> TryFrom<(NamEvent, &'a mut StringCache, &'a mut HashMap<u64, String>)> for Nam<'a> {
    type Error = Error;

    fn try_from(
        value: (NamEvent, &'a mut StringCache, &'a mut HashMap<u64, String>),
    ) -> Result<Self, Self::Error> {
        let event = value.0;
        let cache = value.1;
        let name_map = value.2;

        let bind = &event.name.iter().map(|&c| c as u8).collect::<Vec<u8>>();
        let name = str::from_utf8(&bind)?;
        let name = name.replace('\0', "");
        cache.insert_str(&name)?;
        if name != "" {
            name_map.insert(event.obj, name.clone());
        }

        Ok(Self {
            obj: event.obj,
            id: event.id,
            name: cache.get_str(&name),
        })
    }
}

#[derive(CtfEventClass)]
#[event_name = "PF"]
pub struct Pf {
    pub pfa: u64,
    pub error: u64,
    pub space: u64,
}

impl TryFrom<PfEvent> for Pf {
    type Error = Error;

    fn try_from(value: PfEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            pfa: value.pfa,
            error: value.error,
            space: value.space,
        })
    }
}

#[derive(CtfEventClass)]
#[event_name_from_event_type]
pub struct Unsupported {
    // No payload fields
}

#[repr(i64)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Sequence)]
pub enum TaskState {
    Running = 0,
    Interruptible = 1,
    UnInterruptible = 2,
    Stopped = 4,
    Traced = 8,
    ExitDead = 16,
    ExitZombie = 32,
    Parked = 64,
    Dead = 128,
    WakeKill = 256,
    Waking = 512,
    NoLoad = 1024,
    New = 2048,
}

impl TaskState {
    fn as_ffi(&self) -> *const i8 {
        let ptr = match self {
            TaskState::Running => b"TASK_RUNNING\0".as_ptr(),
            TaskState::Interruptible => b"TASK_INTERRUPTIBLE\0".as_ptr(),
            TaskState::UnInterruptible => b"TASK_UNINTERRUPTIBLE\0".as_ptr(),
            TaskState::Stopped => b"TASK_STOPPED\0".as_ptr(),
            TaskState::Traced => b"TASK_TRACED\0".as_ptr(),
            TaskState::ExitDead => b"EXIT_DEAD\0".as_ptr(),
            TaskState::ExitZombie => b"EXIT_ZOMBIE\0".as_ptr(),
            TaskState::Parked => b"TASK_PARKED\0".as_ptr(),
            TaskState::Dead => b"TASK_DEAD\0".as_ptr(),
            TaskState::WakeKill => b"TASK_WAKEKILL\0".as_ptr(),
            TaskState::Waking => b"TASK_WAKING\0".as_ptr(),
            TaskState::NoLoad => b"TASK_NOLOAD\0".as_ptr(),
            TaskState::New => b"TASK_NEW\0".as_ptr(),
        };
        ptr as *const i8
    }

    fn as_i64(&self) -> i64 {
        *self as i64
    }
}

#[derive(CtfEventClass)]
#[event_name = "sched_switch"]
pub struct SchedSwitch<'a> {
    pub src_event_type: &'a CStr,
    pub prev_comm: &'a CStr,
    pub prev_tid: i64,
    pub prev_prio: i64,
    pub prev_state: TaskState,
    pub next_comm: &'a CStr,
    pub next_tid: i64,
    pub next_prio: i64,
}

impl<'a>
    TryFrom<(
        EventType,
        ContextSwitchEvent,
        &'a mut StringCache,
        &'a mut HashMap<u64, String>,
    )> for SchedSwitch<'a>
{
    type Error = Error;

    // TODO
    fn try_from(
        value: (
            EventType,
            ContextSwitchEvent,
            &'a mut StringCache,
            &'a mut HashMap<u64, String>,
        ),
    ) -> Result<Self, Self::Error> {
        let event_type = value.0;
        let event = value.1;
        let cache = value.2;
        let name_map = value.3;

        let src = event.from_sched;
        let dst = event.dst;

        let mut prev_comm = src.to_string();
        if let Some(s) = name_map.get(&src) {
            prev_comm = s.to_string();
        }

        let mut next_comm = dst.to_string();
        if let Some(d) = name_map.get(&dst) {
            next_comm = d.to_string();
        }

        cache.insert_type(event_type)?;
        cache.insert_str(&src.to_string())?;
        cache.insert_str(&dst.to_string())?;
        cache.insert_str(&prev_comm)?;
        cache.insert_str(&next_comm)?;

        // TODO type casts
        // TODO no comm info in event
        Ok(Self {
            src_event_type: cache.get_type(&event_type),
            prev_comm: cache.get_str(&prev_comm),
            prev_tid: src as i64,
            prev_prio: event.from_prio as i64,
            prev_state: TaskState::Running, // TODO always running?
            next_comm: cache.get_str(&next_comm),
            next_tid: dst as i64,
            next_prio: 9999, // TODO get actual next prio
        })
    }
}
