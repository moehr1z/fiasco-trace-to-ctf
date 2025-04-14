/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "TRAP"]
#[br(little)]
pub struct TrapEvent {
    pub common: EventCommon,

    pub trapno: i8,
    pub __pad_1: [i8; 1],
    pub error: u16,
    pub __pad_2: [i8; 6],
    pub rbp: u64,
    pub cr2: u64,
    pub rax: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub cs: u16,
    pub ds: u16,
}

