#![allow(clippy::manual_c_str_literals)]

use crate::interruptor::Interruptor;
use crate::opts::Opts;
use crate::plugin::{TrcPlugin, TrcPluginState};
use crate::{convert::TrcCtfConverter, types::BorrowedCtfState};
use babeltrace2_sys::{
    ffi, source_plugin_descriptors, BtResult, BtResultExt, CtfPluginSinkFsInitParams,
    EncoderPipeline, Error, LoggingLevel, MessageIteratorStatus, Plugin, RunStatus, SelfComponent,
    SelfMessageIterator, SourcePluginDescriptor, SourcePluginHandler,
};
use chrono::prelude::{DateTime, Utc};
use clap::Parser;
use l4re_traceparse::{event::Event, parser::EventParser};
use std::io::Cursor;
use std::{
    ffi::{CStr, CString},
    fs::File,
    io::BufReader,
    path::PathBuf,
    ptr,
};
use tracing::{debug, error, info, warn};

pub struct Converter {
    pipeline: EncoderPipeline,
}

impl Converter {
    pub fn new(events: Vec<Event>) -> Result<Self, Box<dyn std::error::Error>> {
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
            Box::new(TrcPluginState::new(intr, events, &opts)?);
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
