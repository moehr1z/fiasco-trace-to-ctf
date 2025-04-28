use std::ffi::CStr;

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;
use log::warn;
use num_enum::TryFromPrimitive;

use crate::{converter::types::StringCache, event::ipc::IpcEvent};

const IPCWAIT: u8 = IpcType::OpenWait as u8 | IpcType::Recv as u8;
const IPCENDANDWAIT: u8 = IpcType::OpenWait as u8 | IpcType::Send as u8 | IpcType::Recv as u8;
const IPCREPLYANDWAIT: u8 =
    IpcType::OpenWait as u8 | IpcType::Send as u8 | IpcType::Recv as u8 | IpcType::Reply as u8;
const IPCCALLIPC: u8 = IpcType::Send as u8 | IpcType::Recv as u8;

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
enum IpcType {
    Call = 0,
    Send = 1,
    Recv = 2,
    OpenWait = 4,
    Reply = 8,
    Wait = IPCWAIT,
    SendAndWait = IPCENDANDWAIT,
    ReplyAndWait = IPCREPLYANDWAIT,
    CallIpc = IPCCALLIPC,
}

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
        let event = v.0;
        let rcv_name = v.1;
        let cache = v.2;

        let type_number = (event.dst & 0xf) as u8;
        let type_var: IpcType = type_number.try_into().unwrap_or_else(|_| {
            warn!("Unknown IPC type number {type_number}");
            IpcType::Send
        });
        let type_name = format!("{:?}", type_var).to_string();
        cache.insert_str(&type_name)?;
        let type_ = cache.get_str(&type_name);

        let rcv_name = cache.get_str(rcv_name);

        Ok(Self {
            tag: event.tag,
            dword_1: event.dword[0],
            dword_2: event.dword[1],
            dst: event.dst,
            dbg_id: event.dbg_id,
            label: event.label,
            timeout: event.timeout,
            to_abs_rcv: event.to_abs_rcv,
            rcv_name,
            type_,
        })
    }
}
