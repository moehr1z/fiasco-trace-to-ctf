/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct SchedEvent {
    pub common: EventCommon,

    pub mode: u16,
    pub owner: L4_ktrace_t__Context,
    pub id: u16,
    pub prio: u16,
    pub __pad_1: [i8; 4],
    pub left: i64,
    pub quantum: u64,
}

