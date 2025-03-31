/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use super::common::EventCommon;
use super::typedefs::*;
use binrw::BinRead;

//TODO not yet implemented
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct KeEvent {
    pub common: EventCommon,

}

