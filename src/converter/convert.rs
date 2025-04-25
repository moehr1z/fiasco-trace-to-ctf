use super::types::{BorrowedCtfState, StringCache};
use super::{CTX_MASK, events::*};
use crate::event::{
    Event, common::EventCommon, destroy::DestroyEvent, event_type::EventType,
    factory::FactoryEvent, pf::PfEvent,
};
use crate::helpers;
use babeltrace2_sys::{BtResultExt, Error, ffi};
use log::warn;
use std::collections::{HashMap, hash_map::Entry};
use std::ptr;

// macro to emit basic events which don't require special processing (basically everything which
// uses the CtfEventClass macro)
macro_rules! emit_event {
    ($evt:ty, $conv:ident, $ev:ident, $ctf_state:ident, $event_common:ident) => {{
        let stream_class = unsafe { ffi::bt_stream_borrow_class($ctf_state.stream_mut()) };
        let event_class = $conv.event_class(
            stream_class,
            $event_common.type_.try_into().unwrap(),
            <$evt>::event_class,
        )?;
        let msg = $ctf_state.create_message(event_class, $event_common.tsc);
        let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
        $conv.add_event_common_ctx($event_common, ctf_event)?;
        $ev.emit_event(ctf_event)?;
        $ctf_state.push_message(msg)?;
    }};
}

// TODO remove unused event classes
pub struct TrcCtfConverter {
    unknown_event_class: *mut ffi::bt_event_class,
    user_event_class: *mut ffi::bt_event_class,
    sched_switch_event_class: *mut ffi::bt_event_class,
    sched_migrate_task_event_class: *mut ffi::bt_event_class,
    irq_handler_entry_event_class: *mut ffi::bt_event_class,
    irq_handler_exit_event_class: *mut ffi::bt_event_class,
    sched_wakeup_event_class: *mut ffi::bt_event_class,
    event_classes: HashMap<EventType, *mut ffi::bt_event_class>,
    string_cache: StringCache,
    name_map: HashMap<u64, (String, String)>, // ctx pointer -> (name, dbg_id)
}

impl Drop for TrcCtfConverter {
    fn drop(&mut self) {
        unsafe {
            for (_, event_class) in self.event_classes.drain() {
                ffi::bt_event_class_put_ref(event_class);
            }
            ffi::bt_event_class_put_ref(self.sched_wakeup_event_class);
            ffi::bt_event_class_put_ref(self.irq_handler_entry_event_class);
            ffi::bt_event_class_put_ref(self.irq_handler_exit_event_class);
            ffi::bt_event_class_put_ref(self.sched_switch_event_class);
            ffi::bt_event_class_put_ref(self.sched_migrate_task_event_class);
            ffi::bt_event_class_put_ref(self.user_event_class);
            ffi::bt_event_class_put_ref(self.unknown_event_class);
        }
    }
}

impl TrcCtfConverter {
    pub fn new() -> Self {
        let mut string_cache: StringCache = Default::default();
        string_cache.insert_str("").unwrap(); // TODO error handling

        Self {
            unknown_event_class: ptr::null_mut(),
            user_event_class: ptr::null_mut(),
            sched_switch_event_class: ptr::null_mut(),
            sched_migrate_task_event_class: ptr::null_mut(),
            irq_handler_entry_event_class: ptr::null_mut(),
            irq_handler_exit_event_class: ptr::null_mut(),
            sched_wakeup_event_class: ptr::null_mut(),
            event_classes: Default::default(),
            string_cache,
            name_map: HashMap::new(),
        }
    }

    pub fn create_event_common_context(
        &mut self,
        trace_class: *mut ffi::bt_trace_class,
    ) -> Result<*mut ffi::bt_field_class, Error> {
        unsafe {
            // Create common event context
            // event ID, event count, instruction pointer, timestamp, ctx, pmc1/2, kclock
            let base_event_context = ffi::bt_field_class_structure_create(trace_class);

            let event_id_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            ffi::bt_field_class_integer_set_preferred_display_base(
            event_id_field,
            ffi::bt_field_class_integer_preferred_display_base::BT_FIELD_CLASS_INTEGER_PREFERRED_DISPLAY_BASE_HEXADECIMAL,
        );
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"id".as_ptr() as _,
                event_id_field,
            );
            ret.capi_result()?;

            let event_count_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"event_count".as_ptr() as _,
                event_count_field,
            );
            ret.capi_result()?;

            let event_ip_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            ffi::bt_field_class_integer_set_preferred_display_base(
            event_ip_field,
            ffi::bt_field_class_integer_preferred_display_base::BT_FIELD_CLASS_INTEGER_PREFERRED_DISPLAY_BASE_HEXADECIMAL,
        );
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"ip".as_ptr() as _,
                event_ip_field,
            );
            ret.capi_result()?;

            let timer_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"timer".as_ptr() as _,
                timer_field,
            );
            ret.capi_result()?;

            let event_ctx_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            ffi::bt_field_class_integer_set_preferred_display_base(
            event_ctx_field,
            ffi::bt_field_class_integer_preferred_display_base::BT_FIELD_CLASS_INTEGER_PREFERRED_DISPLAY_BASE_HEXADECIMAL,
        );
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"ctx".as_ptr() as _,
                event_ctx_field,
            );
            ret.capi_result()?;

            let event_pmc1_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"pmc1".as_ptr() as _,
                event_pmc1_field,
            );
            ret.capi_result()?;

            let event_pmc2_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"pmc2".as_ptr() as _,
                event_pmc2_field,
            );
            ret.capi_result()?;

            let event_kclock_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"kclock".as_ptr() as _,
                event_kclock_field,
            );
            ret.capi_result()?;

            let event_name_field = ffi::bt_field_class_string_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"name".as_ptr() as _,
                event_name_field,
            );
            ret.capi_result()?;

            let event_dbg_id_field = ffi::bt_field_class_string_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                c"dbg_id".as_ptr() as _,
                event_dbg_id_field,
            );
            ret.capi_result()?;

            ffi::bt_field_class_put_ref(event_id_field);
            ffi::bt_field_class_put_ref(event_count_field);
            ffi::bt_field_class_put_ref(event_ip_field);
            ffi::bt_field_class_put_ref(timer_field);
            ffi::bt_field_class_put_ref(event_ctx_field);
            ffi::bt_field_class_put_ref(event_pmc1_field);
            ffi::bt_field_class_put_ref(event_pmc2_field);
            ffi::bt_field_class_put_ref(event_kclock_field);
            ffi::bt_field_class_put_ref(event_name_field);
            ffi::bt_field_class_put_ref(event_dbg_id_field);

            Ok(base_event_context)
        }
    }

    /// Create the special event classes upfront, remaining classes will get
    /// created on the fly
    pub fn create_event_classes(&mut self, stream: *mut ffi::bt_stream) -> Result<(), Error> {
        let stream_class = unsafe { ffi::bt_stream_borrow_class(stream) };
        self.sched_switch_event_class = SchedSwitch::event_class(stream_class)?;
        self.sched_switch_event_class = SchedMigrateTask::event_class(stream_class)?;
        Ok(())
    }

    fn add_event_common_ctx(
        &mut self,
        common: EventCommon,
        event: *mut ffi::bt_event,
    ) -> Result<(), Error> {
        let event_id = common.type_;

        unsafe {
            let common_ctx_field = ffi::bt_event_borrow_common_context_field(event);

            let event_id_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 0);
            ffi::bt_field_integer_unsigned_set_value(event_id_field, event_id as u64);

            let event_count_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 1);
            ffi::bt_field_integer_unsigned_set_value(event_count_field, common.number);

            let ip_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 2);
            ffi::bt_field_integer_unsigned_set_value(ip_field, common.ip);

            let timer_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 3);
            ffi::bt_field_integer_unsigned_set_value(timer_field, common.tsc);

            let ctx_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 4);
            ffi::bt_field_integer_unsigned_set_value(ctx_field, common.ctx & CTX_MASK); // TODO

            let pmc1_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 5);
            ffi::bt_field_integer_unsigned_set_value(pmc1_field, common.pmc1 as u64);

            let pmc2_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 6);
            ffi::bt_field_integer_unsigned_set_value(pmc2_field, common.pmc2 as u64);

            let kclock_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 7);
            ffi::bt_field_integer_unsigned_set_value(kclock_field, common.kclock as u64);

            let name_dbg_tuple = self.name_map.get(&(common.ctx & CTX_MASK));

            let name_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 8);
            let c_name = if let Some((n, _)) = name_dbg_tuple {
                self.string_cache.get_str(n)
            } else {
                self.string_cache.get_str("")
            };
            ffi::bt_field_string_set_value(name_field, c_name.as_ptr());

            let dbg_id_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 9);
            let c_dbg_id = if let Some((_, d)) = name_dbg_tuple {
                self.string_cache.get_str(d)
            } else {
                self.string_cache.get_str("")
            };
            ffi::bt_field_string_set_value(dbg_id_field, c_dbg_id.as_ptr());

            Ok(())
        }
    }

    fn event_class<F>(
        &mut self,
        stream_class: *mut ffi::bt_stream_class,
        event_type: EventType,
        f: F,
    ) -> Result<*const ffi::bt_event_class, Error>
    where
        F: FnOnce(*mut ffi::bt_stream_class) -> Result<*mut ffi::bt_event_class, Error>,
    {
        let event_class_ref = match self.event_classes.entry(event_type) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let event_class = f(stream_class)?;
                v.insert(event_class)
            }
        };
        Ok(*event_class_ref as *const _)
    }

    pub fn convert(&mut self, event: Event, ctf_state: &mut BorrowedCtfState) -> Result<(), Error> {
        let event_type = event.event_type();
        let event_common = event.event_common();
        let event_timestamp = event_common.tsc;

        match event {
            Event::Nam(ev) => {
                let mut was_valid = true;
                let name = helpers::i8_array_to_string(ev.name);
                let name = if let Ok(n) = name {
                    n
                } else {
                    // TODO not sure why, but sometimes when you enable IPC events there's some
                    // gibberish in some name fields
                    warn!(
                        "Could not convert Nam event bytes to name string! Not converting this event. (event nr: {}, bytes: {:?})",
                        ev.common.number, ev.name
                    );
                    was_valid = false;
                    "".to_string()
                };

                self.name_map
                    .insert(ev.obj & CTX_MASK, (name.clone(), ev.id.to_string()));
                self.string_cache.insert_str(&name)?;
                self.string_cache.insert_str(&ev.id.to_string())?;

                if was_valid {
                    let stream_class =
                        unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                    let event_class =
                        self.event_class(stream_class, event_type, Nam::event_class)?;
                    let msg = ctf_state.create_message(event_class, event_timestamp);
                    let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                    self.add_event_common_ctx(event_common, ctf_event)?;

                    Nam::try_from((ev, &mut self.string_cache))?.emit_event(ctf_event)?;
                    ctf_state.push_message(msg)?;
                }
            }
            Event::ContextSwitch(ev) => {
                let event_class = self.sched_switch_event_class;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;
                SchedSwitch::try_from((
                    event_type,
                    ev,
                    &mut self.string_cache,
                    &mut self.name_map,
                ))?
                .emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::Migration(ev) => {
                let event_class = self.sched_migrate_task_event_class;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;
                SchedMigrateTask::try_from((ev, &mut self.string_cache, &mut self.name_map))?
                    .emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::Ipc(ev) => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class = self.event_class(stream_class, event_type, Ipc::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;

                // TODO this is slow, use an id -> name map
                let res = self
                    .name_map
                    .iter()
                    .find(|(_, (_name, id))| *id == ev.dbg_id.to_string());
                let rcv_name = if let Some((_, (name, _))) = res {
                    name.to_string()
                } else {
                    "".to_string()
                };
                let rcv_name = self.string_cache.get_str(&rcv_name);

                Ipc::try_from((ev, rcv_name))?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::IpcRes(ev) => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class =
                    self.event_class(stream_class, event_type, IpcRes::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;
                IpcRes::try_from(ev)?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }

            Event::Destroy(ev) => {
                self.name_map.remove(&(ev.obj & CTX_MASK));
                emit_event!(DestroyEvent, self, ev, ctf_state, event_common)
            }
            Event::Factory(ev) => {
                self.name_map
                    .insert(ev.newo & CTX_MASK, ("".to_string(), ev.id.to_string()));
                self.string_cache.insert_str(&ev.id.to_string())?;
                emit_event!(FactoryEvent, self, ev, ctf_state, event_common)
            }

            Event::Pf(ev) => emit_event!(PfEvent, self, ev, ctf_state, event_common),

            // The rest are named events with no payload
            _ => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class = self.event_class(stream_class, event_type, |stream_class| {
                    Unsupported::event_class(event_type, stream_class)
                })?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;
                Unsupported {}.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
        }

        Ok(())
    }
}
