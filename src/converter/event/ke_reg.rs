use std::ffi::CStr;

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{converter::types::StringCache, event::ke_reg::KeRegEvent, helpers};

#[derive(CtfEventClass)]
#[event_name = "KEREG"]
pub struct KeReg<'a> {
    pub v0: u64,
    pub v1: u64,
    pub v2: u64,
    pub msg: &'a CStr,
}

impl<'a> TryFrom<(KeRegEvent, &'a mut StringCache)> for KeReg<'a> {
    type Error = Error;

    fn try_from(value: (KeRegEvent, &'a mut StringCache)) -> Result<Self, Self::Error> {
        let (event, cache) = value;
        let msg = &helpers::i8_array_to_string(event.msg)?;
        let id = cache.insert_str(msg)?;

        Ok(Self {
            v0: event.v[0],
            v1: event.v[1],
            v2: event.v[2],
            msg: cache.get_str_by_id(id),
        })
    }
}
