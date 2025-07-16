use std::{cell::RefCell, collections::HashMap, ffi::CStr, rc::Rc};

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{
    converter::{
        CTX_MASK,
        kernel_object::{KernelObject, ThreadState},
        types::StringCache,
    },
    event::ipc_res::IpcResEvent,
};

use super::ipc_type::IpcType;

#[derive(CtfEventClass)]
#[event_name = "IPCRES"]
pub struct IpcRes<'a> {
    have_snd: u8,
    is_np: u8,
    tag: u64,
    dword_1: u64,
    dword_2: u64,
    result: u64,
    from: u64,
    dst: u64,
    pair_event: u64,
    type_: &'a CStr,
}

impl<'a>
    TryFrom<(
        IpcResEvent,
        &'a mut StringCache,
        &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
    )> for IpcRes<'a>
{
    type Error = Error;

    fn try_from(
        v: (
            IpcResEvent,
            &'a mut StringCache,
            &'a mut Rc<RefCell<HashMap<u64, KernelObject>>>,
        ),
    ) -> Result<Self, Self::Error> {
        let (event, cache, map) = v;

        if let Some(o) = map.borrow_mut().get_mut(&(event.common.ctx & CTX_MASK)) {
            if let KernelObject::Thread(t) = o {
                t.state = ThreadState::Running;
            }
        }

        let type_name = IpcType::num_to_str((event.dst & 0xf) as u8);
        cache.insert_str(&type_name)?;
        let type_ = cache.get_str(&type_name);

        Ok(Self {
            have_snd: event.have_snd,
            is_np: event.is_np,
            tag: event.tag,
            dword_1: event.dword[0],
            dword_2: event.dword[1],
            result: event.result,
            from: event.from,
            dst: event.dst,
            pair_event: event.pair_event,
            type_,
        })
    }
}
