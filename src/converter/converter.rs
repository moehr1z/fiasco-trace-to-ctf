#![allow(clippy::manual_c_str_literals)]

use super::interruptor::Interruptor;
use super::opts::Opts;
use super::plugin::{TrcPlugin, TrcPluginState};
use crate::parser::event::Event;
use crate::parser::event::typedefs::L4Addr;
use babeltrace2_sys::{
    CtfPluginSinkFsInitParams, EncoderPipeline, LoggingLevel, RunStatus, SourcePluginHandler,
};
use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString};
use tracing::debug;

pub struct Converter {
    pipeline: EncoderPipeline,
}

impl Converter {
    pub fn new(
        events: VecDeque<Event>,
        name_db: HashMap<L4Addr, Vec<(String, Option<u64>)>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let opts = Opts {
            clock_name: "monotonic".to_string(),
            trace_name: "l4re".to_string(),
            log_level: LoggingLevel::Warn,
            output: "ctf_trace".into(),
            input: "test".into(),
        };

        let intr = Interruptor::new();
        let intr_clone = intr.clone();
        ctrlc::set_handler(move || {
            if intr_clone.is_set() {
                let exit_code = if cfg!(target_family = "unix") {
                    // 128 (fatal error signal "n") + 2 (control-c is fatal error signal 2)
                    130
                } else {
                    // Windows code 3221225786
                    // -1073741510 == C000013A
                    -1073741510
                };
                std::process::exit(exit_code);
            }

            debug!("Shutdown signal received");
            intr_clone.set();
        })?;

        let output_path = CString::new(opts.output.to_str().unwrap())?;
        let params = CtfPluginSinkFsInitParams::new(
            Some(true), // assume_single_trace
            None,       // ignore_discarded_events
            None,       // ignore_discarded_packets
            Some(true), // quiet
            &output_path,
        )?;

        let state_inner: Box<dyn SourcePluginHandler> =
            Box::new(TrcPluginState::new(intr, events, &opts, name_db)?);
        let state = Box::new(state_inner);

        let mut pipeline = EncoderPipeline::new::<TrcPlugin>(opts.log_level, state, &params)?;

        return Ok(Self { pipeline });
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
}
