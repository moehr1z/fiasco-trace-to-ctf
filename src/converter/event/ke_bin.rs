use std::ffi::CStr;

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{converter::types::StringCache, event::ke_bin::KeBinEvent, helpers};

#[derive(CtfEventClass)]
#[event_name = "KEBIN"]
pub struct KeBin<'a> {
    pub msg: &'a CStr,
}

impl<'a> TryFrom<(KeBinEvent, &'a mut StringCache)> for KeBin<'a> {
    type Error = Error;

    fn try_from(value: (KeBinEvent, &'a mut StringCache)) -> Result<Self, Self::Error> {
        let (event, cache) = value;
        let msg = &helpers::i8_array_to_string(event.msg)?;
        let id = cache.insert_str(msg)?;

        Ok(Self {
            msg: cache.get_str_by_id(id),
        })
    }
}
