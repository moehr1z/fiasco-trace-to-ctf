/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "GATE"]
#[br(little)]
pub struct GateEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub gate_dbg_id: u64,
    pub thread_dbg_id: u64,
    pub label: u64,
}

