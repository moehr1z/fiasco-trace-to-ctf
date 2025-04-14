/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "SVM"]
#[br(little)]
pub struct SvmEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub exitcode: u64,
    pub exitinfo1: u64,
    pub exitinfo2: u64,
    pub rip: u64,
}

