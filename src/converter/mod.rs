mod convert;
mod event;
pub mod interruptor;
mod plugin;
mod types;

use crate::event::Event;
use crate::opts::Opts;
use babeltrace2_sys::{CtfPluginSinkFsInitParams, EncoderPipeline, RunStatus, SourcePluginHandler};
use interruptor::Interruptor;
use plugin::{TrcPlugin, TrcPluginState};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::rc::Rc;

const CTX_MASK: u64 = 0xFFFFFFFFFFFFF000;

pub struct Converter {
    pipeline: EncoderPipeline,
}

impl Converter {
    pub fn new(
        events: Rc<RefCell<VecDeque<Event>>>,
        eof_signal: Rc<Cell<bool>>,
        opts: Opts,
        cpu_id: u8,
        intr: Interruptor,
        name_map: Rc<RefCell<HashMap<u64, (String, String)>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let output_path = CString::new(opts.output.to_str().unwrap())?;
        let params = CtfPluginSinkFsInitParams::new(
            Some(true), // assume_single_trace
            None,       // ignore_discarded_events
            None,       // ignore_discarded_packets
            Some(true), // quiet
            &output_path,
        )?;

        let state_inner: Box<dyn SourcePluginHandler> = Box::new(TrcPluginState::new(
            intr, events, &opts, eof_signal, cpu_id, name_map,
        )?);
        let state = Box::new(state_inner);

        let pipeline = EncoderPipeline::new::<TrcPlugin>(opts.log_level, state, &params)?;

        Ok(Self { pipeline })
    }

    pub fn convert(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let run_status = self.pipeline.graph.run_once()?;
            if RunStatus::End == run_status {
                break;
            }
        }
        Ok(())
    }

    pub fn convert_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.pipeline.graph.run_once()?;
        Ok(())
    }
}
