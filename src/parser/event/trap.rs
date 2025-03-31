/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use super::common::EventCommon;
use super::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct TrapEvent {
    pub common: EventCommon,

    pub trapno: i8,
    pub __pad_1: [i8; 1],
    pub error: L4_ktrace_t__Unsigned16,
    pub __pad_2: [i8; 6],
    pub rbp: L4_ktrace_t__Mword,
    pub cr2: L4_ktrace_t__Mword,
    pub rax: L4_ktrace_t__Mword,
    pub rflags: L4_ktrace_t__Mword,
    pub rsp: L4_ktrace_t__Mword,
    pub cs: L4_ktrace_t__Unsigned16,
    pub ds: L4_ktrace_t__Unsigned16,
}

