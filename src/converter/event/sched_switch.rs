use crate::converter::CTX_MASK;
use crate::converter::kernel_object::KernelObject;
use crate::converter::types::StringCache;
use crate::event::context_switch::ContextSwitchEvent;
use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;
use enum_iterator::Sequence;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::rc::Rc;

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
        ContextSwitchEvent,
        &'a mut StringCache,
        &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
    )> for SchedSwitch<'a>
{
    type Error = Error;

    fn try_from(
        value: (
            ContextSwitchEvent,
            &'a mut StringCache,
            &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
        ),
    ) -> Result<Self, Self::Error> {
        let (event, cache, kernel_object_map) = value;

        let src = event.common.ctx & CTX_MASK;
        let dst = event.dst & CTX_MASK;

        let mut prev_tid: i64 = src as i64;
        let prev_comm_id = if let Some(o) = kernel_object_map.borrow().get(&src) {
            let dbg_id = o.id();
            let name = o.name();

            if let Ok(tid_i64) = dbg_id.parse() {
                prev_tid = tid_i64
            }
            if !name.is_empty() {
                cache.insert_str(name)?
            } else {
                cache.insert_str(dbg_id)?
            }
        } else {
            cache.insert_str(&src.to_string())?
        };

        let mut next_tid: i64 = dst as i64;

        let next_comm_id = if let Some(o) = kernel_object_map.borrow().get(&dst) {
            let dbg_id = o.id();
            let name = o.name();

            if let Ok(tid_i64) = dbg_id.parse() {
                next_tid = tid_i64
            }
            if !name.is_empty() {
                cache.insert_str(name)?
            } else {
                cache.insert_str(dbg_id)?
            }
        } else {
            cache.insert_str(&dst.to_string())?
        };

        Ok(Self {
            prev_comm: cache.get_str_by_id(prev_comm_id),
            prev_tid,
            prev_prio: event.from_prio as i64,
            prev_state: TaskState::Running, // TODO always running?
            next_comm: cache.get_str_by_id(next_comm_id),
            next_tid,
            next_prio: 9999, // TODO get actual next prio
        })
    }
}
