use super::CTX_MASK;
use super::event::ipc::Ipc;
use super::event::ipc_res::IpcRes;
use super::event::ke_bin::KeBin;
use super::event::ke_reg::KeReg;
use super::event::nam::Nam;
use super::event::sched_migrate_task::SchedMigrateTask;
use super::event::sched_switch::SchedSwitch;
use super::kernel_object::{BaseKernelObject, KernelObject};
use super::types::{BorrowedCtfState, StringCache};
use crate::converter::event::ke::Ke;
use crate::event::bp::BpEvent;
use crate::event::drq::DrqEvent;
use crate::event::empty::EmptyEvent;
use crate::event::exregs::ExregsEvent;
use crate::event::fullsize::FullsizeEvent;
use crate::event::gate::GateEvent;
use crate::event::ieh::IehEvent;
use crate::event::ipfh::IpfhEvent;
use crate::event::irq::IrqEvent;
use crate::event::rcu::RcuEvent;
use crate::event::sched::SchedEvent;
use crate::event::svm::SvmEvent;
use crate::event::timer::TimerEvent;
use crate::event::tmap::TmapEvent;
use crate::event::trap::TrapEvent;
use crate::event::vcpu::VcpuEvent;
use crate::event::{
    Event, common::EventCommon, destroy::DestroyEvent, factory::FactoryEvent, pf::PfEvent,
};
use crate::helpers;
use babeltrace2_sys::{BtResultExt, Error, ffi};
use log::info;
use std::cell::RefCell;
use std::collections::{HashMap, hash_map::Entry};
use std::ptr;
use std::rc::Rc;

// macro to emit basic events which don't require special processing (basically everything which
// uses the CtfEventClass macro)
macro_rules! emit_event {
    ($ev_name:ident, $evt:ty, $conv:ident, $ev:ident, $ctf_state:ident, $event_common:ident) => {{
        let stream_class = unsafe { ffi::bt_stream_borrow_class($ctf_state.stream_mut()) };
        let event_class = $conv.event_class(stream_class, $ev_name, <$evt>::event_class)?;
        let msg = $ctf_state.create_message(event_class, $event_common.tsc);
        let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
        $conv.add_event_common_ctx($event_common, ctf_event)?;
        $ev.emit_event(ctf_event)?;
        $ctf_state.push_message(msg)?;
    }};
}

pub struct TrcCtfConverter {
    sched_switch_event_class: *mut ffi::bt_event_class,
    sched_migrate_task_event_class: *mut ffi::bt_event_class,
    event_classes: HashMap<String, *mut ffi::bt_event_class>,
    string_cache: StringCache,
    kernel_object_map: Rc<RefCell<HashMap<u64, KernelObject>>>,
}

impl Drop for TrcCtfConverter {
    fn drop(&mut self) {
        unsafe {
            for (_, event_class) in self.event_classes.drain() {
                ffi::bt_event_class_put_ref(event_class);
            }
            ffi::bt_event_class_put_ref(self.sched_switch_event_class);
            ffi::bt_event_class_put_ref(self.sched_migrate_task_event_class);
        }
    }
}

impl TrcCtfConverter {
    pub fn new(kernel_object_map: Rc<RefCell<HashMap<u64, KernelObject>>>) -> Self {
        let mut string_cache: StringCache = Default::default();
        string_cache.insert_str("").unwrap();

        Self {
            sched_switch_event_class: ptr::null_mut(),
            sched_migrate_task_event_class: ptr::null_mut(),
            event_classes: Default::default(),
            string_cache,
            kernel_object_map,
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
        self.sched_migrate_task_event_class = SchedMigrateTask::event_class(stream_class)?;
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
            ffi::bt_field_integer_unsigned_set_value(ctx_field, common.ctx & CTX_MASK);

            let pmc1_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 5);
            ffi::bt_field_integer_unsigned_set_value(pmc1_field, common.pmc1 as u64);

            let pmc2_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 6);
            ffi::bt_field_integer_unsigned_set_value(pmc2_field, common.pmc2 as u64);

            let kclock_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 7);
            ffi::bt_field_integer_unsigned_set_value(kclock_field, common.kclock as u64);

            let map = self.kernel_object_map.borrow();
            let kernel_object = map.get(&(common.ctx & CTX_MASK));

            let (c_name_id, c_dbg_id_id) = match kernel_object {
                Some(o) => {
                    let id_1 = self.string_cache.insert_str(o.name())?;
                    let id_2 = self.string_cache.insert_str(o.id())?;
                    (id_1, id_2)
                }
                None => {
                    let id_1 = self.string_cache.insert_str("")?;
                    (id_1, id_1)
                }
            };

            let c_name = self.string_cache.get_str_by_id(c_name_id);
            let c_dbg_id = self.string_cache.get_str_by_id(c_dbg_id_id);

            let name_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 8);
            ffi::bt_field_string_set_value(name_field, c_name.as_ptr());

            let dbg_id_field =
                ffi::bt_field_structure_borrow_member_field_by_index(common_ctx_field, 9);
            ffi::bt_field_string_set_value(dbg_id_field, c_dbg_id.as_ptr());

            Ok(())
        }
    }

    fn event_class<F>(
        &mut self,
        stream_class: *mut ffi::bt_stream_class,
        event_type: String,
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
        let event_type = event.to_string();
        let event_common = event.event_common();
        let event_timestamp = event_common.tsc;

        match event {
            Event::Ke(ev) => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class = self.event_class(stream_class, event_type, Ke::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;

                Ke::try_from((ev, &mut self.string_cache))?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::KeReg(ev) => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class = self.event_class(stream_class, event_type, KeReg::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;

                KeReg::try_from((ev, &mut self.string_cache))?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::KeBin(ev) => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class = self.event_class(stream_class, event_type, KeBin::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;

                KeBin::try_from((ev, &mut self.string_cache))?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::Nam(ev) => {
                let name = helpers::i8_array_to_string(ev.name);
                let name = if let Ok(n) = name {
                    n
                } else {
                    // TODO not sure why, but sometimes when you enable IPC events there's some
                    // gibberish in some name fields
                    info!(
                        "Could not convert Nam event bytes to name string! (event nr: {}, bytes: {:?})",
                        ev.common.number, ev.name
                    );
                    "".to_string()
                };

                let pointer = ev.obj & CTX_MASK;
                let id = ev.id.to_string();
                match self.kernel_object_map.borrow_mut().entry(pointer) {
                    Entry::Occupied(mut entry) => {
                        let obj = entry.get_mut();
                        obj.set_id(id);
                        obj.set_name(name);
                    }
                    Entry::Vacant(entry) => {
                        let new_obj = KernelObject::Generic(BaseKernelObject {
                            id,
                            name: name.to_string(),
                        });
                        entry.insert(new_obj);
                    }
                }

                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class = self.event_class(stream_class, event_type, Nam::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;

                Nam::try_from((ev, &mut self.string_cache))?.emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::ContextSwitch(ev) => {
                let event_class = self.sched_switch_event_class;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;
                SchedSwitch::try_from((ev, &mut self.string_cache, &mut self.kernel_object_map))?
                    .emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::Migration(ev) => {
                let event_class = self.sched_migrate_task_event_class;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;
                SchedMigrateTask::try_from((
                    ev,
                    &mut self.string_cache,
                    &mut self.kernel_object_map,
                ))?
                .emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::Ipc(ev) => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class = self.event_class(stream_class, event_type, Ipc::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;

                Ipc::try_from((ev, &mut self.string_cache, &mut self.kernel_object_map))?
                    .emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::IpcRes(ev) => {
                let stream_class = unsafe { ffi::bt_stream_borrow_class(ctf_state.stream_mut()) };
                let event_class =
                    self.event_class(stream_class, event_type, IpcRes::event_class)?;
                let msg = ctf_state.create_message(event_class, event_timestamp);
                let ctf_event = unsafe { ffi::bt_message_event_borrow_event(msg) };
                self.add_event_common_ctx(event_common, ctf_event)?;
                IpcRes::try_from((ev, &mut self.string_cache, &mut self.kernel_object_map))?
                    .emit_event(ctf_event)?;
                ctf_state.push_message(msg)?;
            }
            Event::Destroy(ev) => {
                self.kernel_object_map
                    .borrow_mut()
                    .remove(&(ev.obj & CTX_MASK));
                emit_event!(event_type, DestroyEvent, self, ev, ctf_state, event_common)
            }
            Event::Factory(ev) => {
                let id = ev.newo.to_string();
                if id == "0" {
                    println!("CREATED OBJ WITH ID 0");
                }
                let name = "".to_string();
                let new_obj = KernelObject::Generic(BaseKernelObject { id, name });

                self.kernel_object_map
                    .borrow_mut()
                    .insert(ev.obj & CTX_MASK, new_obj);
                emit_event!(event_type, FactoryEvent, self, ev, ctf_state, event_common)
            }
            Event::Pf(ev) => emit_event!(event_type, PfEvent, self, ev, ctf_state, event_common),
            Event::Drq(ev) => emit_event!(event_type, DrqEvent, self, ev, ctf_state, event_common),
            Event::Vcpu(ev) => {
                emit_event!(event_type, VcpuEvent, self, ev, ctf_state, event_common)
            }
            Event::Gate(ev) => {
                emit_event!(event_type, GateEvent, self, ev, ctf_state, event_common)
            }
            Event::Irq(ev) => emit_event!(event_type, IrqEvent, self, ev, ctf_state, event_common),
            Event::Rcu(ev) => emit_event!(event_type, RcuEvent, self, ev, ctf_state, event_common),
            Event::Tmap(ev) => {
                emit_event!(event_type, TmapEvent, self, ev, ctf_state, event_common)
            }
            Event::Bp(ev) => emit_event!(event_type, BpEvent, self, ev, ctf_state, event_common),
            Event::Empty(ev) => {
                emit_event!(event_type, EmptyEvent, self, ev, ctf_state, event_common)
            }
            Event::Sched(ev) => {
                emit_event!(event_type, SchedEvent, self, ev, ctf_state, event_common)
            }
            Event::Trap(ev) => {
                emit_event!(event_type, TrapEvent, self, ev, ctf_state, event_common)
            }
            Event::Fullsize(ev) => {
                emit_event!(event_type, FullsizeEvent, self, ev, ctf_state, event_common)
            }
            Event::Ieh(ev) => emit_event!(event_type, IehEvent, self, ev, ctf_state, event_common),
            Event::Ipfh(ev) => {
                emit_event!(event_type, IpfhEvent, self, ev, ctf_state, event_common)
            }
            Event::Exregs(ev) => {
                emit_event!(event_type, ExregsEvent, self, ev, ctf_state, event_common)
            }
            Event::Timer(ev) => {
                emit_event!(event_type, TimerEvent, self, ev, ctf_state, event_common)
            }
            Event::Svm(ev) => emit_event!(event_type, SvmEvent, self, ev, ctf_state, event_common),
        }

        Ok(())
    }
}
