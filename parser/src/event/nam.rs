/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct NamEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub obj: L4_ktrace_t__Kobject,
    pub id: L4_ktrace_t__Mword,
    pub name: [i8; 32], 
}

