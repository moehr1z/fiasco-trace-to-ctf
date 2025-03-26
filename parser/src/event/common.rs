/* Note, automatically generated from Fiasco binary */

use crate::event::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct EventCommon {
    pub number: L4_ktrace_t__Mword,
    pub ip: L4_ktrace_t__Address,
    pub tsc: L4_ktrace_t__Unsigned64,
    pub ctx: L4_ktrace_t__Context,
    pub pmc1: L4_ktrace_t__Unsigned32,
    pub pmc2: L4_ktrace_t__Unsigned32,
    pub kclock: L4_ktrace_t__Unsigned32,
    pub type_: L4_ktrace_t__Unsigned8,
    pub cpu: L4_ktrace_t__Unsigned8,
}

