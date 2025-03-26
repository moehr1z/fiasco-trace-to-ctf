/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use crate::event::common::EventCommon;
use crate::event::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct SvmEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub exitcode: L4_ktrace_t__Mword,
    pub exitinfo1: L4_ktrace_t__Mword,
    pub exitinfo2: L4_ktrace_t__Mword,
    pub rip: L4_ktrace_t__Mword,
}

