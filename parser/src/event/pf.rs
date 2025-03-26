/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct PfEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub pfa: L4_ktrace_t__Address,
    pub error: L4_ktrace_t__Mword,
    pub space: L4_ktrace_t__Space,
}

