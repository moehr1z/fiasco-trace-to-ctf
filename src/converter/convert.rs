use super::events::*;
use super::types::{BorrowedCtfState, StringCache};
use crate::parser::event::Event;
use crate::parser::event::common::EventCommon;
use crate::parser::event::event_type::EventType;
use crate::parser::event::typedefs::L4Addr;
use babeltrace2_sys::{BtResultExt, Error, ffi};
use std::collections::{HashMap, hash_map::Entry};
use std::ptr;

// TODO remove unused event classes
pub struct TrcCtfConverter {
    unknown_event_class: *mut ffi::bt_event_class,
    user_event_class: *mut ffi::bt_event_class,
    sched_switch_event_class: *mut ffi::bt_event_class,
    irq_handler_entry_event_class: *mut ffi::bt_event_class,
    irq_handler_exit_event_class: *mut ffi::bt_event_class,
    sched_wakeup_event_class: *mut ffi::bt_event_class,
    event_classes: HashMap<EventType, *mut ffi::bt_event_class>,
    string_cache: StringCache,
    name_map: HashMap<L4Addr, Vec<(String, Option<u64>)>>,
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
            ffi::bt_event_class_put_ref(self.user_event_class);
            ffi::bt_event_class_put_ref(self.unknown_event_class);
        }
    }
}

impl TrcCtfConverter {
    pub fn new(name_map: HashMap<L4Addr, Vec<(String, Option<u64>)>>) -> Self {
        Self {
            unknown_event_class: ptr::null_mut(),
            user_event_class: ptr::null_mut(),
            sched_switch_event_class: ptr::null_mut(),
            irq_handler_entry_event_class: ptr::null_mut(),
            irq_handler_exit_event_class: ptr::null_mut(),
            sched_wakeup_event_class: ptr::null_mut(),
            event_classes: Default::default(),
            string_cache: Default::default(),
            name_map,
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
                b"id\0".as_ptr() as _,
                event_id_field,
            );
            ret.capi_result()?;

            let event_count_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                b"event_count\0".as_ptr() as _,
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
                b"ip\0".as_ptr() as _,
                event_ip_field,
            );
            ret.capi_result()?;

            let timer_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                b"timer\0".as_ptr() as _,
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
                b"ctx\0".as_ptr() as _,
                event_ctx_field,
            );
            ret.capi_result()?;

            let event_pmc1_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                b"pmc1\0".as_ptr() as _,
                event_pmc1_field,
            );
            ret.capi_result()?;

            let event_pmc2_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                b"pmc2\0".as_ptr() as _,
                event_pmc2_field,
            );
            ret.capi_result()?;

            let event_kclock_field = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                base_event_context,
                b"kclock\0".as_ptr() as _,
                event_kclock_field,
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

            Ok(base_event_context)
        }
    }

    /// Create the special event classes upfront, remaining classes will get
    /// created on the fly
    pub fn create_event_classes(&mut self, stream: *mut ffi::bt_stream) -> Result<(), Error> {
        let stream_class = unsafe { ffi::bt_stream_borrow_class(stream) };
        self.sched_switch_event_class = SchedSwitch::event_class(stream_class)?;
        Ok(())
    }

    fn add_event_common_ctx(
        &mut self,
        event_id: u8,
        common: EventCommon,
        event: *mut ffi::bt_event,
    ) -> Result<(), Error> {
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
            ffi::bt_field_integer_unsigned_set_value(ctx_field, common.ctx);

            let pmc1_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 5);
            ffi::bt_field_integer_unsigned_set_value(pmc1_field, common.pmc1 as u64);

            let pmc2_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 6);
            ffi::bt_field_integer_unsigned_set_value(pmc2_field, common.pmc2 as u64);

            let kclock_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 7);
            ffi::bt_field_integer_unsigned_set_value(kclock_field, common.kclock as u64);

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
        let event_id: u8 = event.event_type().into();
        let event_timestamp = event_common.tsc;

        let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };

        match event {
            Event::Nam(ev) => {
                let event_class = self.event_class(stream_class, event_type, Nam::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_id, event_common, ctf_event)?;
                Nam::try_from((ev, &mut self.string_cache))?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::Pf(ev) => {
                let event_class = self.event_class(stream_class, event_type, Pf::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_id, event_common, ctf_event)?;
                Pf::try_from(ev)?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::ContextSwitch(ev) => {
                let event_class = self.sched_switch_event_class;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_id, event_common, ctf_event)?;
                SchedSwitch::try_from((
                    event_type,
                    ev,
                    &mut self.string_cache,
                    &mut self.name_map,
                ))?
                .emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            // The rest are named events with no payload
            _ => {
                let event_class = self.event_class(stream_class, event_type, |stream_class| {
                    Unsupported::event_class(event_type, stream_class)
                })?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_id, event_common, ctf_event)?;
                Unsupported {}.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
        }

        Ok(())
    }
}
