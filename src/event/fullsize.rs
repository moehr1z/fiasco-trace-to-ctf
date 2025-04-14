/* Note, automatically generated from Fiasco binary */

#[allow(unused_imports)]
use ctf_macros::CtfEventClass;

use super::common::EventCommon;
use binrw::BinRead;
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]
#[event_name = "FULLSIZE"]
#[br(little)]
pub struct FullsizeEvent {
    pub common: EventCommon,

    pub padding: [i8; 80], 
}

