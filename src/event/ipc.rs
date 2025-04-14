/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct IpcEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub tag: u64,
    pub dword: [u64; 2], 
    pub dst: u64,
    pub dbg_id: u64,
    pub label: u64,
    pub timeout: u32,
    pub __pad_1: [i8; 4],
    pub to_abs_rcv: u64,
}

