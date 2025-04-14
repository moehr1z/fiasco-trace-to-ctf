/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "TMAP"]
#[br(little)]
pub struct TmapEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub id: u64,
    pub mask: u64,
    pub fpage: u64,
    pub map: u8,
}

