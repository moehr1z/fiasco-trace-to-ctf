/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct FactoryEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub op: L4_ktrace_t__Smword,
    pub buffer: L4_ktrace_t__Cap_index,
    pub id: L4_ktrace_t__Mword,
    pub ram: L4_ktrace_t__Mword,
    pub newo: L4_ktrace_t__Mword,
}

