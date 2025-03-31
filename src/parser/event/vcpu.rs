/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use super::common::EventCommon;
use super::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct VcpuEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub state: L4_ktrace_t__Mword,
    pub ip: L4_ktrace_t__Mword,
    pub sp: L4_ktrace_t__Mword,
    pub space: L4_ktrace_t__Mword,
    pub err: L4_ktrace_t__Mword,
    pub type_: u8,
    pub trap: u8,
}

