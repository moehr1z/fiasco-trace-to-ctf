/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use super::common::EventCommon;
use super::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct BpEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub address: L4_ktrace_t__Address,
    pub len: i32,
    pub __pad_1: [i8; 4],
    pub value: L4_ktrace_t__Mword,
    pub mode: i32,
}
