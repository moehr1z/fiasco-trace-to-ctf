use std::{collections::HashMap, ffi::CStr};

use babeltrace2_sys::Error;
use ctf_macros::CtfEventClass;

use crate::{
    converter::{CTX_MASK, types::StringCache},
    event::migration::MigrationEvent,
};

#[derive(CtfEventClass)]
#[event_name = "sched_migrate_task"]
pub struct SchedMigrateTask<'a> {
    pub comm: &'a CStr,
    pub tid: i64,
    pub prio: i64,
    pub orig_cpu: i32,
    pub dest_cpu: i32,
}

impl<'a>
    TryFrom<(
        MigrationEvent,
        &'a mut StringCache,
        &'a mut HashMap<u64, (String, String)>,
    )> for SchedMigrateTask<'a>
{
    type Error = Error;

    fn try_from(
        value: (
            MigrationEvent,
            &'a mut StringCache,
            &'a mut HashMap<u64, (String, String)>,
        ),
    ) -> Result<Self, Self::Error> {
        let event = value.0;
        let cache = value.1;
        let name_map = value.2;

        let ctx = event.common.ctx & CTX_MASK;
        let mut comm = ctx.to_string();
        let mut tid = ctx as i64;
        if let Some((name, dbg_id)) = name_map.get(&ctx) {
            if !name.is_empty() {
                comm = name.clone();
            } else {
                comm = dbg_id.clone();
            }
            if let Ok(tid_i64) = dbg_id.parse() {
                tid = tid_i64
            }
        }

        cache.insert_str(&comm)?;

        Ok(Self {
            comm: cache.get_str(&comm),
            tid,
            prio: 0, // TODO
            orig_cpu: event.src_cpu as i32,
            dest_cpu: event.target_cpu as i32,
        })
    }
}
