/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "PF"]
#[br(little)]
pub struct PfEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub pfa: u64,
    pub error: u64,
    pub space: u64,
}

