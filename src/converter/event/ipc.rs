use std::{cell::RefCell, collections::HashMap, ffi::CStr, rc::Rc};

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{
    converter::{
        CTX_MASK,
        kernel_object::{KernelObject, ThreadState},
        types::StringCache,
    },
    event::ipc::IpcEvent,
};

use super::ipc_type::IpcType;

#[derive(CtfEventClass)]
#[event_name = "IPC"]
pub struct Ipc<'a> {
    tag: u64,
    dword_1: u64,
    dword_2: u64,
    dst: u64,
    dbg_id: u64,
    label: u64,
    timeout: u32,
    to_abs_rcv: u64,
    rcv_name: &'a CStr,
    type_: &'a CStr,
    dst_thread_name: &'a CStr,
}

impl<'a>
    TryFrom<(
        IpcEvent,
        &'a mut StringCache,
        &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
    )> for Ipc<'a>
{
    type Error = Error;

    fn try_from(
        v: (
            IpcEvent,
            &'a mut StringCache,
            &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
        ),
    ) -> Result<Self, Self::Error> {
        let (event, cache, map) = v;

        if let Some(o) = map.borrow_mut().get_mut(&(event.common.ctx & CTX_MASK)) {
            if let KernelObject::Thread(t) = o {
                t.state = ThreadState::Blocked;
            }
        }

        // TODO: this is slow, use an id -> name map
        let binding = map.borrow();
        let res = binding
            .iter()
            .find(|(_, o)| *o.id() == event.dbg_id.to_string());
        let mut rcv_name = "";
        let mut dst_thread_name = "";
        if let Some((_, o)) = res {
            rcv_name = o.name();

            if let KernelObject::Gate(g) = o {
                if let Some(k) = binding.get(&g.thread) {
                    dst_thread_name = if k.name() != "" { k.name() } else { k.id() }
                }
            }
        }

        let type_name = IpcType::num_to_str((event.dst & 0xf) as u8);
        cache.insert_str(&type_name)?;
        cache.insert_str(rcv_name)?;
        cache.insert_str(dst_thread_name)?;

        // if let Some(dst_thread) = binding.get()

        Ok(Self {
            tag: event.tag,
            dword_1: event.dword[0],
            dword_2: event.dword[1],
            dst: event.dst,
            dbg_id: event.dbg_id,
            label: event.label,
            timeout: event.timeout,
            to_abs_rcv: event.to_abs_rcv,
            rcv_name: cache.get_str(rcv_name),
            type_: cache.get_str(&type_name),
            dst_thread_name: cache.get_str(dst_thread_name),
        })
    }
}
