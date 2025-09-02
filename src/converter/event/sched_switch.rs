use crate::converter::CTX_MASK;
use crate::converter::kernel_object::{BaseKernelObject, KernelObject, ThreadObject, ThreadState};
use crate::converter::types::StringCache;
use crate::event::context_switch::ContextSwitchEvent;
use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;
use enum_iterator::Sequence;
use log::error;
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

impl From<ThreadState> for TaskState {
    fn from(state: ThreadState) -> Self {
        match state {
            ThreadState::Running => TaskState::Running,
            ThreadState::Blocked => TaskState::Stopped,
        }
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

// TODO: this is pretty convoluted as of now and should be simplified sometime
impl<'a>
    TryFrom<(
        ContextSwitchEvent,
        &'a mut StringCache,
        &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
        &'a mut Option<ThreadObject>,
    )> for SchedSwitch<'a>
{
    type Error = Error;

    fn try_from(
        value: (
            ContextSwitchEvent,
            &'a mut StringCache,
            &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
            &'a mut Option<ThreadObject>,
        ),
    ) -> Result<Self, Self::Error> {
        let (event, cache, kernel_object_map, last_sched_in) = value;

        let src = event.common.ctx & CTX_MASK;
        let dst = event.dst & CTX_MASK;

        let mut prev_tid: i64 = src as i64;
        let mut prev_state = TaskState::Running;

        // handle thread of scheduling context
        // src may or may not be the same as the sched context
        if let Some(o) = kernel_object_map
            .borrow_mut()
            .get_mut(&(event.from_sched & CTX_MASK))
        {
            let prio = event.from_prio;
            let id;
            let name;
            if prio == 0 {
                id = "0".to_string();
                name = format!("idle {}", event.common.cpu).to_string();
            } else {
                id = o.id().to_string();
                name = o.name().to_string();
            }

            match o {
                KernelObject::Generic(_) => {
                    let new_obj = KernelObject::Thread(ThreadObject {
                        base: BaseKernelObject { id, name },
                        state: ThreadState::Blocked,
                        prio,
                    });
                    *o = new_obj;
                }
                KernelObject::Thread(t) => {
                    t.prio = prio;
                    t.base.id = id;
                    t.base.name = name;
                }
                _ => {
                    error!("Sched switch on none thread object");
                    return Err(Error::PluginError("Non thread kernel object".to_string()));
                }
            }
        }

        // if the src kernel object is not of type thread yet make it so
        if let Some(o) = kernel_object_map.borrow_mut().get_mut(&src) {
            let prio = event.from_prio;
            let id = o.id().to_string();
            let name = o.name().to_string();

            if let KernelObject::Generic(_) = o {
                let new_obj = KernelObject::Thread(ThreadObject {
                    base: BaseKernelObject { id, name },
                    state: ThreadState::Blocked,
                    prio,
                });
                *o = new_obj;
            }
        }

        let mut prev_prio = event.from_prio;
        let prev_comm_id = if let Some(o) = kernel_object_map.borrow_mut().get_mut(&src) {
            if let KernelObject::Thread(t) = o {
                prev_prio = t.prio;
                prev_state = t.state.into();
                let mut dbg_id = o.id();
                let mut name = o.name();

                // last dst is not the same as this ones src, so we entered an exluded thread,
                // which switched to another excluded thread (not traced), which switched back to
                // some not excluded thread
                if let Some(last_thread) = last_sched_in {
                    if last_thread.base.id != dbg_id {
                        prev_prio = last_thread.prio;
                        prev_state = last_thread.state.into();
                        dbg_id = &last_thread.base.id;
                        name = &last_thread.base.name;
                    }
                }

                if let Ok(tid_i64) = dbg_id.parse() {
                    prev_tid = tid_i64
                }

                if !name.is_empty() {
                    cache.insert_str(name)?
                } else {
                    cache.insert_str(dbg_id)?
                }
            } else {
                error!("sched_switch on a non thread kernel object!");
                return Err(Error::PluginError("Non thread kernel object".to_string()));
            }
        } else {
            cache.insert_str(&src.to_string())?
        };

        let mut next_tid: i64 = dst as i64;

        // if the dst kernel object is not of type thread yet make it so
        if let Some(o) = kernel_object_map.borrow_mut().get_mut(&dst) {
            if let KernelObject::Generic(_) = o {
                let new_obj = KernelObject::Thread(ThreadObject {
                    base: BaseKernelObject {
                        id: o.id().to_string(),
                        name: o.name().to_string(),
                    },
                    state: ThreadState::Blocked,
                    prio: 1000,
                });
                *o = new_obj;
            }
        }

        let mut next_prio = 1000;
        let next_comm_id = if let Some(o) = kernel_object_map.borrow_mut().get_mut(&dst) {
            if let KernelObject::Thread(t) = o {
                *last_sched_in = Some(t.clone());

                next_prio = t.prio;
                t.state = ThreadState::Running;
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
                error!("sched_switch on a non thread kernel object!");
                return Err(Error::PluginError("Non thread kernel object".to_string()));
            }
        } else {
            cache.insert_str(&dst.to_string())?
        };

        Ok(Self {
            prev_comm: cache.get_str_by_id(prev_comm_id),
            prev_tid,
            prev_prio: prev_prio.try_into().unwrap(),
            prev_state,
            next_comm: cache.get_str_by_id(next_comm_id),
            next_tid,
            next_prio: next_prio.try_into().unwrap(),
        })
    }
}
