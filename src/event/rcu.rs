/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "RCU"]
#[br(little)]
pub struct RcuEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub cpu: u32,
    pub __pad_1: [i8; 4],
    pub item: u64,
    pub cb: u64,
    pub event: u8,
}

