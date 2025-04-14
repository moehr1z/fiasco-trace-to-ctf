/* Note, automatically generated from Fiasco binary */

use binrw::BinRead;

#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct EventCommon {
    pub number: u64,
    pub ip: u64,
    pub tsc: u64,
    pub ctx: u64,
    pub pmc1: u32,
    pub pmc2: u32,
    pub kclock: u32,
    pub type_: u8,
    pub cpu: u8,
}

