/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct ExregsEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub id: L4_ktrace_t__Mword,
    pub ip: L4_ktrace_t__Mword,
    pub sp: L4_ktrace_t__Mword,
    pub op: L4_ktrace_t__Mword,
}

