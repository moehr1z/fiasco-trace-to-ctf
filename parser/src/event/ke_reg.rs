/* Note, automatically generated from Fiasco binary */

#![allow(unused_imports)]
use crate::event::common::EventCommon;
use crate::event::typedefs::*;
use binrw::BinRead;

//TODO not yet implemented
#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct KeRegEvent {
    pub common: EventCommon,

}

