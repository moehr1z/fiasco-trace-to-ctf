/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use super::common::EventCommon;
use super::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct ContextSwitchEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub dst: L4_ktrace_t__Context,
    pub dst_orig: L4_ktrace_t__Context,
    pub kernel_ip: L4_ktrace_t__Address,
    pub lock_cnt: L4_ktrace_t__Mword,
    pub from_space: L4_ktrace_t__Space,
    pub from_sched: L4_ktrace_t__Sched_context,
    pub from_prio: L4_ktrace_t__Mword,
}

