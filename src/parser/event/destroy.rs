/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use super::common::EventCommon;
use super::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct DestroyEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub obj: L4_ktrace_t__Kobject,
    pub id: L4_ktrace_t__Mword,
    pub type_: L4_ktrace_t__cxx__Type_info,
    pub ram: L4_ktrace_t__Mword,
}

