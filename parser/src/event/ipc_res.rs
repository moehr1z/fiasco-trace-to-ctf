/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use crate::event::common::EventCommon;
use crate::event::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct IpcResEvent {
    pub common: EventCommon,

    pub have_snd: L4_ktrace_t__Unsigned8,
    pub is_np: L4_ktrace_t__Unsigned8,
    pub tag: L4_ktrace_t__L4_msg_tag,
    pub dword: [L4_ktrace_t__Mword; 2], 
    pub result: L4_ktrace_t__L4_error,
    pub from: L4_ktrace_t__Mword,
    pub dst: L4_ktrace_t__L4_obj_ref,
    pub pair_event: L4_ktrace_t__Mword,
}

