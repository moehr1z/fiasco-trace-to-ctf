use crate::converter::CTX_MASK;
use crate::converter::types::StringCache;
use crate::event::{context_switch::ContextSwitchEvent, event_type::EventType};
use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;
use enum_iterator::Sequence;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CStr;

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
        &'a mut HashMap<u64, (String, String)>,
    )> for SchedSwitch<'a>
{
    type Error = Error;

    fn try_from(
        value: (
            EventType,
            ContextSwitchEvent,
            &'a mut StringCache,
            &'a mut HashMap<u64, (String, String)>,
        ),
    ) -> Result<Self, Self::Error> {
        let event_type = value.0;
        let event = value.1;
        let cache = value.2;
        let name_map = value.3;

        let src = event.common.ctx & CTX_MASK;
        let dst = event.dst & CTX_MASK;

        let mut prev_comm = src.to_string();
        let mut prev_tid: i64 = src as i64;
        if let Some((name, dbg_id)) = name_map.get(&src) {
            if !name.is_empty() {
                prev_comm = name.clone();
            } else {
                prev_comm = dbg_id.clone();
            }
            if let Ok(tid_i64) = dbg_id.parse() {
                prev_tid = tid_i64
            }
        }

        let mut next_comm = dst.to_string();
        let mut next_tid: i64 = dst as i64;
        if let Some((name, dbg_id)) = name_map.get(&dst) {
            if !name.is_empty() {
                next_comm = name.clone();
            } else {
                next_comm = dbg_id.clone();
            }
            if let Ok(tid_i64) = dbg_id.parse() {
                next_tid = tid_i64
            }
        }

        cache.insert_type(event_type)?;
        cache.insert_str(&src.to_string())?;
        cache.insert_str(&dst.to_string())?;
        cache.insert_str(&prev_comm)?;
        cache.insert_str(&next_comm)?;

        Ok(Self {
            src_event_type: cache.get_type(&event_type),
            prev_comm: cache.get_str(&prev_comm),
            prev_tid,
            prev_prio: event.from_prio as i64,
            prev_state: TaskState::Running, // TODO always running?
            next_comm: cache.get_str(&next_comm),
            next_tid,
            next_prio: 9999, // TODO get actual next prio
        })
    }
}
