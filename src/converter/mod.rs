mod convert;
mod event;
mod interruptor;
pub mod opts;
mod plugin;
mod types;

use crate::event::Event;
use babeltrace2_sys::{CtfPluginSinkFsInitParams, EncoderPipeline, RunStatus, SourcePluginHandler};
use interruptor::Interruptor;
use opts::Opts;
use plugin::{TrcPlugin, TrcPluginState};
use std::collections::VecDeque;
use std::ffi::CString;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use tracing::debug;

const CTX_MASK: u64 = 0xFFFFFFFFFFFFF000;

pub struct Converter {
    pipeline: EncoderPipeline,
}

impl Converter {
    pub fn new(
        events: Arc<Mutex<VecDeque<Event>>>,
        eof_signal: Arc<AtomicBool>,
        opts: Opts,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
            Box::new(TrcPluginState::new(intr, events, &opts, eof_signal)?);
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
