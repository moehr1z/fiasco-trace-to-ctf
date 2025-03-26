/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use crate::event::common::EventCommon;
use crate::event::typedefs::*;
use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct TimerEvent {
    pub common: EventCommon,

    pub __pre_pad: [i8; 2],
    pub user_ip: L4_ktrace_t__Address,
}

