use super::types::StringCache;
use crate::event::{
    context_switch::ContextSwitchEvent, destroy::DestroyEvent, event_type::EventType,
    factory::FactoryEvent, nam::NamEvent,
};
use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;
use enum_iterator::Sequence;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::str;

// TODO - any way to use serde-reflection to synthesize these?

#[allow(dead_code)] // TODO
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

#[allow(dead_code)] // TODO
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

impl<'a> TryFrom<(NamEvent, &'a mut StringCache)> for Nam<'a> {
    type Error = Error;

    fn try_from(value: (NamEvent, &'a mut StringCache)) -> Result<Self, Self::Error> {
        let event = value.0;
        let cache = value.1;

        let bind = &event.name.iter().map(|&c| c as u8).collect::<Vec<u8>>();
        let name = str::from_utf8(bind)?;
        let name = name.replace('\0', "");
        cache.insert_str(&name)?;

        Ok(Self {
            obj: event.obj,
            id: event.id,
            name: cache.get_str(&name),
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
        match self {
            TaskState::Running => c"TASK_RUNNING".as_ptr(),
            TaskState::Interruptible => c"TASK_INTERRUPTIBLE".as_ptr(),
            TaskState::UnInterruptible => c"TASK_UNINTERRUPTIBLE".as_ptr(),
            TaskState::Stopped => c"TASK_STOPPED".as_ptr(),
            TaskState::Traced => c"TASK_TRACED".as_ptr(),
            TaskState::ExitDead => c"EXIT_DEAD".as_ptr(),
            TaskState::ExitZombie => c"EXIT_ZOMBIE".as_ptr(),
            TaskState::Parked => c"TASK_PARKED".as_ptr(),
            TaskState::Dead => c"TASK_DEAD".as_ptr(),
            TaskState::WakeKill => c"TASK_WAKEKILL".as_ptr(),
            TaskState::Waking => c"TASK_WAKING".as_ptr(),
            TaskState::NoLoad => c"TASK_NOLOAD".as_ptr(),
            TaskState::New => c"TASK_NEW".as_ptr(),
        }
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
        &'a mut HashMap<u64, Vec<(String, Option<u64>)>>,
    )> for SchedSwitch<'a>
{
    type Error = Error;

    // TODO
    fn try_from(
        value: (
            EventType,
            ContextSwitchEvent,
            &'a mut StringCache,
            &'a mut HashMap<u64, Vec<(String, Option<u64>)>>,
        ),
    ) -> Result<Self, Self::Error> {
        let event_type = value.0;
        let event = value.1;
        let ts = event.common.tsc;
        let cache = value.2;
        let name_map = value.3;

        let src = event.from_sched;
        let dst = event.dst;

        let src = src & 0xFFFFFFFFFFFFF000; // TODO
        let dst = dst & 0xFFFFFFFFFFFFF000; // TODO

        let mut prev_comm = src.to_string();
        if let Some(s) = name_map.get(&src) {
            for (name, valid_until) in s {
                if valid_until.is_none() || ts < valid_until.unwrap() {
                    prev_comm = name.clone();
                    break;
                }
            }
        }

        let mut next_comm = dst.to_string();
        if let Some(d) = name_map.get(&dst) {
            for (name, valid_until) in d {
                if valid_until.is_none() || ts < valid_until.unwrap() {
                    next_comm = name.clone();
                    break;
                }
            }
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
