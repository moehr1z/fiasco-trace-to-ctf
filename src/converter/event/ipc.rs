use std::ffi::CStr;

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::event::ipc::IpcEvent;

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
}

impl<'a> TryFrom<(IpcEvent, &'a CStr)> for Ipc<'a> {
    type Error = Error;

    fn try_from(v: (IpcEvent, &'a CStr)) -> Result<Self, Self::Error> {
        let value = v.0;
        let rcv_name = v.1;

        Ok(Self {
            tag: value.tag,
            dword_1: value.dword[0],
            dword_2: value.dword[1],
            dst: value.dst,
            dbg_id: value.dbg_id,
            label: value.label,
            timeout: value.timeout,
            to_abs_rcv: value.to_abs_rcv,
            rcv_name,
        })
    }
}
