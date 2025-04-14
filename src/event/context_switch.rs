/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "CONTEXTSWITCH"]
#[br(little)]
pub struct ContextSwitchEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub dst: u64,
    pub dst_orig: u64,
    pub kernel_ip: u64,
    pub lock_cnt: u64,
    pub from_space: u64,
    pub from_sched: u64,
    pub from_prio: u64,
}

