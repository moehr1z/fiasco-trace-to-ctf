/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "IEH"]
#[br(little)]
pub struct IehEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub cap_idx: u64,
}

