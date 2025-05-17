use super::interruptor::Interruptor;
use super::{convert::TrcCtfConverter, types::BorrowedCtfState};
use crate::event::Event;
use crate::opts::Opts;
use babeltrace2_sys::{
    BtResult, BtResultExt, Error, MessageIteratorStatus, Plugin, SelfComponent,
    SelfMessageIterator, SourcePluginDescriptor, SourcePluginHandler, ffi,
    source_plugin_descriptors,
};
use chrono::prelude::{DateTime, Utc};
use log::error;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::{
    ffi::{CStr, CString},
    ptr,
};
use tracing::debug;

pub struct TrcPluginState {
    interruptor: Interruptor,
    events: Arc<Mutex<VecDeque<Event>>>,
    clock_name: CString,
    trace_name: CString,
    trace_creation_time: DateTime<Utc>,
    first_event_observed: bool,
    eof_reached: Arc<AtomicBool>,
    stream_is_open: bool,
    stream: *mut ffi::bt_stream,
    packet: *mut ffi::bt_packet,
    cpu_id: u8,
    converter: TrcCtfConverter,
}

impl TrcPluginState {
    pub fn new(
        interruptor: Interruptor,
        events: Arc<Mutex<VecDeque<Event>>>,
        opts: &Opts,
        eof_signal: Arc<AtomicBool>,
        cpu_id: u8,
    ) -> Result<Self, Error> {
        let clock_name = CString::new(opts.clock_name.as_str())?;
        let trace_name = CString::new(opts.trace_name.as_str())?;
        Ok(Self {
            interruptor,
            events,
            clock_name,
            trace_name,
            trace_creation_time: Utc::now(),
            first_event_observed: false,
            eof_reached: eof_signal,
            stream_is_open: false,
            // NOTE: timestamp/event trackers get re-initialized on the first event
            stream: ptr::null_mut(),
            packet: ptr::null_mut(),
            cpu_id,
            converter: TrcCtfConverter::new(),
        })
    }

    pub fn create_metadata_and_stream_objects(
        &mut self,
        mut component: SelfComponent,
    ) -> Result<(), Error> {
        unsafe {
            let trace_class = ffi::bt_trace_class_create(component.inner_mut());
            ffi::bt_trace_class_set_assigns_automatic_stream_class_id(trace_class, 0);

            // Create common event context
            let base_event_context = self.converter.create_event_common_context(trace_class)?;

            // Setup the default clock class
            let clock_class = ffi::bt_clock_class_create(component.inner_mut());
            let ret =
                ffi::bt_clock_class_set_name(clock_class, self.clock_name.as_c_str().as_ptr());
            ret.capi_result()?;
            ffi::bt_clock_class_set_origin_is_unix_epoch(clock_class, 0);

            let stream_class = ffi::bt_stream_class_create_with_id(trace_class, self.cpu_id as u64);
            ffi::bt_stream_class_set_default_clock_class(stream_class, clock_class);
            ffi::bt_stream_class_set_supports_packets(
                stream_class,
                1, //supports_packets
                0, // with_beginning_default_clock_snapshot
                0, // with_end_default_clock_snapshot
            );
            ffi::bt_stream_class_set_supports_discarded_packets(
                stream_class,
                0, // supports_discarded_packets
                0, // with_default_clock_snapshots
            );
            ffi::bt_stream_class_set_supports_discarded_events(
                stream_class,
                1, // supports_discarded_events
                0, // with_default_clock_snapshots
            );
            let ret = ffi::bt_stream_class_set_event_common_context_field_class(
                stream_class,
                base_event_context,
            );
            ret.capi_result()?;

            // Add cpu_id packet context
            let packet_context_fc = ffi::bt_field_class_structure_create(trace_class);
            let cpu_id_fc = ffi::bt_field_class_integer_unsigned_create(trace_class);
            let ret = ffi::bt_field_class_structure_append_member(
                packet_context_fc,
                c"cpu_id".as_ptr() as _,
                cpu_id_fc,
            );
            ret.capi_result()?;
            let ret = ffi::bt_stream_class_set_packet_context_field_class(
                stream_class,
                packet_context_fc,
            );
            ret.capi_result()?;
            ffi::bt_field_class_put_ref(cpu_id_fc);
            ffi::bt_field_class_put_ref(packet_context_fc);

            let trace = ffi::bt_trace_create(trace_class);
            ffi::bt_trace_set_name(trace, self.trace_name.as_c_str().as_ptr());

            self.stream = ffi::bt_stream_create(stream_class, trace);
            self.create_new_packet(self.cpu_id.into())?;

            // Put the references we don't need anymore
            ffi::bt_trace_put_ref(trace);
            ffi::bt_clock_class_put_ref(clock_class);
            ffi::bt_stream_class_put_ref(stream_class);
            ffi::bt_trace_class_put_ref(trace_class as *const _);
            ffi::bt_field_class_put_ref(base_event_context);
        }

        Ok(())
    }

    pub fn set_trace_env(&mut self) -> Result<(), Error> {
        unsafe {
            let trace = ffi::bt_stream_borrow_trace(self.stream);
            let ret = ffi::bt_trace_set_environment_entry_string(
                trace,
                c"hostname".as_ptr() as _,
                c"l4re_trace".as_ptr() as _,
            );
            ret.capi_result()?;
            let ret = ffi::bt_trace_set_environment_entry_string(
                trace,
                c"domain".as_ptr() as _,
                c"kernel".as_ptr() as _,
            );
            ret.capi_result()?;
            let ret = ffi::bt_trace_set_environment_entry_string(
                trace,
                c"tracer_name".as_ptr() as _,
                c"lttng-modules".as_ptr() as _,
            );
            ret.capi_result()?;
            let val = CString::new(format!(
                "{}",
                self.trace_creation_time.format("%Y%m%dT%H%M%S+0000")
            ))?;
            let ret = ffi::bt_trace_set_environment_entry_string(
                trace,
                c"trace_creation_datetime".as_ptr() as _,
                val.as_c_str().as_ptr(),
            );
            ret.capi_result()?;
            let val = CString::new(format!("{}", self.trace_creation_time))?;
            let ret = ffi::bt_trace_set_environment_entry_string(
                trace,
                c"trace_creation_datetime_utc".as_ptr() as _,
                val.as_c_str().as_ptr(),
            );
            ret.capi_result()?;
        }
        Ok(())
    }

    pub fn create_new_packet(&mut self, cpu_id: u64) -> Result<(), Error> {
        unsafe {
            if !self.packet.is_null() {
                ffi::bt_packet_put_ref(self.packet);
            }

            self.packet = ffi::bt_packet_create(self.stream);

            let packet_ctx_f = ffi::bt_packet_borrow_context_field(self.packet);
            let cpu_id_f = ffi::bt_field_structure_borrow_member_field_by_index(packet_ctx_f, 0);

            ffi::bt_field_integer_unsigned_set_value(cpu_id_f, cpu_id);
        }
        Ok(())
    }

    pub fn read_event(&mut self) -> Result<Option<Event>, Error> {
        let mut events = self.events.lock().unwrap_or_else(|_| {
            error!("Poisoned lock!");
            panic!()
        });
        if let Some(event) = events.pop_front() {
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    pub fn process_event(
        &mut self,
        event: Event,
        ctf_state: &mut BorrowedCtfState,
    ) -> Result<(), Error> {
        if !self.first_event_observed {
            self.first_event_observed = true;
        }

        self.converter.convert(event, ctf_state)?;

        Ok(())
    }
}

impl SourcePluginHandler for TrcPluginState {
    fn initialize(&mut self, component: SelfComponent) -> Result<(), Error> {
        self.create_metadata_and_stream_objects(component)?;
        self.set_trace_env()?;

        assert!(!self.stream.is_null());
        self.converter.create_event_classes(self.stream)?;

        Ok(())
    }

    fn finalize(&mut self, _component: SelfComponent) -> Result<(), Error> {
        unsafe {
            assert!(!self.packet.is_null());
            ffi::bt_packet_put_ref(self.packet);
            self.packet = ptr::null_mut();

            assert!(!self.stream.is_null());
            ffi::bt_stream_put_ref(self.stream);
            self.stream = ptr::null_mut();
        }

        Ok(())
    }

    fn iterator_next(
        &mut self,
        msg_iter: SelfMessageIterator,
        messages: &mut [*const ffi::bt_message],
    ) -> Result<MessageIteratorStatus, Error> {
        assert!(!self.stream.is_null());

        let mut ctf_state = BorrowedCtfState::new(self.stream, self.packet, msg_iter, messages);

        if self.interruptor.is_set() & !self.eof_reached.load(Relaxed) {
            debug!("Early shutdown");
            self.eof_reached.store(true, Relaxed);

            // Add packet end message
            let msg = unsafe {
                ffi::bt_message_packet_end_create(ctf_state.message_iter_mut(), self.packet)
            };
            ctf_state.push_message(msg)?;

            // Add stream end message
            let msg = unsafe {
                ffi::bt_message_stream_end_create(ctf_state.message_iter_mut(), self.stream)
            };
            ctf_state.push_message(msg)?;

            return Ok(ctf_state.release());
        }

        match self.read_event()? {
            Some(event) => {
                if !self.stream_is_open {
                    debug!("Opening stream");
                    self.stream_is_open = true;

                    // Add stream begin message
                    let msg = unsafe {
                        ffi::bt_message_stream_beginning_create(
                            ctf_state.message_iter_mut(),
                            self.stream,
                        )
                    };
                    ctf_state.push_message(msg)?;

                    // Add packet begin message
                    let msg = unsafe {
                        ffi::bt_message_packet_beginning_create(
                            ctf_state.message_iter_mut(),
                            self.packet,
                        )
                    };
                    ctf_state.push_message(msg)?;
                }

                // TODO need to put_ref(msg) on this and/or all of the msgs?
                self.process_event(event, &mut ctf_state)?;

                Ok(ctf_state.release())
            }
            None => {
                if !self.stream_is_open && self.first_event_observed {
                    // Last iteration can't have messages
                    Ok(MessageIteratorStatus::Done)
                } else if self.eof_reached.load(Relaxed) {
                    debug!("End of file reached");
                    self.eof_reached.store(true, Relaxed);

                    // Add packet end message
                    let msg = unsafe {
                        ffi::bt_message_packet_end_create(ctf_state.message_iter_mut(), self.packet)
                    };
                    ctf_state.push_message(msg)?;

                    // Add stream end message
                    let msg = unsafe {
                        ffi::bt_message_stream_end_create(ctf_state.message_iter_mut(), self.stream)
                    };
                    ctf_state.push_message(msg)?;

                    self.stream_is_open = false;

                    Ok(ctf_state.release())
                } else {
                    Ok(MessageIteratorStatus::NoMessages)
                }
            }
        }
    }
}

pub struct TrcPlugin;

impl SourcePluginDescriptor for TrcPlugin {
    /// Provides source.trace-recorder.output
    const PLUGIN_NAME: &'static [u8] = b"trace-recorder\0";
    const OUTPUT_COMP_NAME: &'static [u8] = b"output\0";
    const GRAPH_NODE_NAME: &'static [u8] = b"source.trace-recorder.output\0";

    fn load() -> BtResult<Plugin> {
        let name = Self::plugin_name();
        Plugin::load_from_statics_by_name(name)
    }

    fn plugin_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::PLUGIN_NAME) }
    }

    fn output_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::OUTPUT_COMP_NAME) }
    }

    fn graph_node_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::GRAPH_NODE_NAME) }
    }
}

source_plugin_descriptors!(TrcPlugin);

pub mod utils_plugin_descriptors {
    use babeltrace2_sys::ffi::*;

    #[link(
        name = "babeltrace-plugin-utils",
        kind = "static",
        modifiers = "+whole-archive"
    )]
    unsafe extern "C" {
        pub static __bt_plugin_descriptor_auto_ptr: *const __bt_plugin_descriptor;
    }
}

pub mod ctf_plugin_descriptors {
    use babeltrace2_sys::ffi::*;

    #[link(
        name = "babeltrace-plugin-ctf",
        kind = "static",
        modifiers = "+whole-archive"
    )]
    unsafe extern "C" {
        pub static __bt_plugin_descriptor_auto_ptr: *const __bt_plugin_descriptor;
    }
}
