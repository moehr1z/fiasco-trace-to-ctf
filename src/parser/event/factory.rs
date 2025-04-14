/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "FACTORY"]
#[br(little)]
pub struct FactoryEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub op: i64,
    pub buffer: u64,
    pub id: u64,
    pub ram: u64,
    pub newo: u64,
}

