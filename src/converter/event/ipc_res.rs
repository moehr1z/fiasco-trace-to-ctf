use std::ffi::CStr;

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{converter::types::StringCache, event::ipc_res::IpcResEvent};

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

impl<'a> TryFrom<(IpcResEvent, &'a mut StringCache)> for IpcRes<'a> {
    type Error = Error;

    fn try_from(v: (IpcResEvent, &'a mut StringCache)) -> Result<Self, Self::Error> {
        let event = v.0;
        let cache = v.1;

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
