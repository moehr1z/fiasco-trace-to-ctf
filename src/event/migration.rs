/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "MIGRATION"]
#[br(little)]
pub struct MigrationEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub state: u64,
    pub user_ip: u64,
    pub src_cpu: u32,
    pub target_cpu: u32,
}

