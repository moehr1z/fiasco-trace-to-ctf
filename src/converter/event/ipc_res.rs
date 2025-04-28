use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::event::ipc_res::IpcResEvent;

#[derive(CtfEventClass)]
#[event_name = "IPCRES"]
pub struct IpcRes {
    have_snd: u8,
    is_np: u8,
    tag: u64,
    dword_1: u64,
    dword_2: u64,
    result: u64,
    from: u64,
    dst: u64,
    pair_event: u64,
}

impl TryFrom<IpcResEvent> for IpcRes {
    type Error = Error;

    fn try_from(value: IpcResEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            have_snd: value.have_snd,
            is_np: value.is_np,
            tag: value.tag,
            dword_1: value.dword[0],
            dword_2: value.dword[1],
            result: value.result,
            from: value.from,
            dst: value.dst,
            pair_event: value.pair_event,
        })
    }
}
