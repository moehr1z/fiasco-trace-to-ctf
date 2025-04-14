/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "EXREGS"]
#[br(little)]
pub struct ExregsEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub id: u64,
    pub ip: u64,
    pub sp: u64,
    pub op: u64,
}

