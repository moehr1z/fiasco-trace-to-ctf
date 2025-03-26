/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct IrqEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub obj: L4_ktrace_t__Irq_base,
    pub chip: L4_ktrace_t__Irq_chip,
    pub pin: L4_ktrace_t__Mword,
}

