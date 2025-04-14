/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "BP"]
#[br(little)]
pub struct BpEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub address: u64,
    pub len: i32,
    pub __pad_1: [i8; 4],
    pub value: u64,
    pub mode: i32,
}

