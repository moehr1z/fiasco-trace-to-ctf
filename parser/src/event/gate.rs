/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use crate::event::common::EventCommon;
use crate::event::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct GateEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub gate_dbg_id: L4_ktrace_t__Mword,
    pub thread_dbg_id: L4_ktrace_t__Mword,
    pub label: L4_ktrace_t__Mword,
}

