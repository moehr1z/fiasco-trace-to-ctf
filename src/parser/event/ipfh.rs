/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use super::common::EventCommon;
use super::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct IpfhEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub pfa: L4_ktrace_t__Mword,
    pub cap_idx: L4_ktrace_t__Cap_index,
    pub err: L4_ktrace_t__Mword,
}

