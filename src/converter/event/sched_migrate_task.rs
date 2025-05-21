use std::{cell::RefCell, collections::HashMap, ffi::CStr, rc::Rc};

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
        &'a mut Rc<RefCell<HashMap<u64, (String, String)>>>,
    )> for SchedMigrateTask<'a>
{
    type Error = Error;

    fn try_from(
        value: (
            MigrationEvent,
            &'a mut StringCache,
            &'a mut Rc<RefCell<HashMap<u64, (String, String)>>>,
        ),
    ) -> Result<Self, Self::Error> {
        let (event, cache, name_map) = value;

        let ctx = event.common.ctx & CTX_MASK;
        let mut tid = ctx as i64;

        let comm_id = if let Some((name, dbg_id)) = name_map.borrow().get(&ctx) {
            if let Ok(tid_i64) = dbg_id.parse() {
                tid = tid_i64
            }

            if !name.is_empty() {
                cache.insert_str(name)?
            } else {
                cache.insert_str(dbg_id)?
            }
        } else {
            cache.insert_str(&ctx.to_string())?
        };

        Ok(Self {
            comm: cache.get_str_by_id(comm_id),
            tid,
            prio: 0, // TODO
            orig_cpu: event.src_cpu as i32,
            dest_cpu: event.target_cpu as i32,
        })
    }
}
