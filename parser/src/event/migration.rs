/* Note, automatically generated from Fiasco binary */

use super::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct MigrationEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub state: L4_ktrace_t__Mword,
    pub user_ip: L4_ktrace_t__Address,
    pub src_cpu: L4_ktrace_t__Cpu_number,
    pub target_cpu: L4_ktrace_t__Cpu_number,
}

