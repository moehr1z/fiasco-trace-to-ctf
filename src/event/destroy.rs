/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "DESTROY"]
#[br(little)]
pub struct DestroyEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub obj: u64,
    pub id: u64,
    pub type_: u64,
    pub ram: u64,
}

