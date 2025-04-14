/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "SCHED"]
#[br(little)]
pub struct SchedEvent {
    pub common: EventCommon,

    pub mode: u16,
    pub owner: u64,
    pub id: u16,
    pub prio: u16,
    pub __pad_1: [i8; 4],
    pub left: i64,
    pub quantum: u64,
}

