use std::ffi::CStr;

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{converter::types::StringCache, event::nam::NamEvent, helpers};

#[derive(CtfEventClass)]
#[event_name = "NAM"]
pub struct Nam<'a> {
    pub obj: u64,
    pub id: u64,
    pub name: &'a CStr,
}

impl<'a> TryFrom<(NamEvent, &'a mut StringCache)> for Nam<'a> {
    type Error = Error;

    fn try_from(value: (NamEvent, &'a mut StringCache)) -> Result<Self, Self::Error> {
        let (event, cache) = value;
        let name = &helpers::i8_array_to_string(event.name)?;
        cache.insert_str(name)?;

        Ok(Self {
            obj: event.obj,
            id: event.id,
            name: cache.get_str(name),
        })
    }
}
