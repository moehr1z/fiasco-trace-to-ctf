/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct DrqEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub func: L4Addr,
    pub thread: L4_ktrace_t__Context,
    pub rq: L4_ktrace_t__Context__Drq,
    pub target_cpu: L4_ktrace_t__Cpu_number,
    pub type_: L4_ktrace_t__Context__Drq_log__Type,
    pub wait: u8,
}

