/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use crate::event::common::EventCommon;
use crate::event::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct IpcEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub tag: L4_ktrace_t__L4_msg_tag,
    pub dword: [L4_ktrace_t__Mword; 2], 
    pub dst: L4_ktrace_t__L4_obj_ref,
    pub dbg_id: L4_ktrace_t__Mword,
    pub label: L4_ktrace_t__Mword,
    pub timeout: L4_ktrace_t__L4_timeout_pair,
    pub __pad_1: [i8; 4],
    pub to_abs_rcv: L4_ktrace_t__Unsigned64,
}

