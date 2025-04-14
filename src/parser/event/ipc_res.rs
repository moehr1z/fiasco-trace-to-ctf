/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct IpcResEvent {
    pub common: EventCommon,

    pub have_snd: u8,
    pub is_np: u8,
    pub tag: u64,
    pub dword: [u64; 2], 
    pub result: u64,
    pub from: u64,
    pub dst: u64,
    pub pair_event: u64,
}

