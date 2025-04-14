/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "VCPU"]
#[br(little)]
pub struct VcpuEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub state: u64,
    pub ip: u64,
    pub sp: u64,
    pub space: u64,
    pub err: u64,
    pub type_: u8,
    pub trap: u8,
}

