/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "DRQ"]
#[br(little)]
pub struct DrqEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub func: u64,
    pub thread: u64,
    pub rq: u64,
    pub target_cpu: u32,
    pub type_: u32,
    pub wait: u8,
}

