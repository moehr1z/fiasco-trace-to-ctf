/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct RcuEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub cpu: L4_ktrace_t__Cpu_number,
    pub __pad_1: [i8; 4],
    pub item: L4_ktrace_t__Rcu_item,
    pub cb: L4Addr,
    pub event: u8,
}

