use std::ffi::CStr;

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{converter::types::StringCache, event::ipc::IpcEvent};

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
}

impl<'a> TryFrom<(IpcEvent, &'a String, &'a mut StringCache)> for Ipc<'a> {
    type Error = Error;

    fn try_from(v: (IpcEvent, &'a String, &'a mut StringCache)) -> Result<Self, Self::Error> {
        let (event, rcv_name, cache) = v;

        let type_name = IpcType::num_to_str((event.dst & 0xf) as u8);
        cache.insert_str(&type_name)?;
        cache.insert_str(rcv_name)?;

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
        })
    }
}
