fn main() {}
// #![allow(clippy::manual_c_str_literals)]
//
// use crate::{convert::TrcCtfConverter, types::BorrowedCtfState};
// use babeltrace2_sys::{
//     ffi, source_plugin_descriptors, BtResult, BtResultExt, CtfPluginSinkFsInitParams,
//     EncoderPipeline, Error, LoggingLevel, MessageIteratorStatus, Plugin, RunStatus, SelfComponent,
//     SelfMessageIterator, SourcePluginDescriptor, SourcePluginHandler,
// };
// use chrono::prelude::{DateTime, Utc};
// use clap::Parser;
// use interruptor::Interruptor;
// use l4re_traceparse::{event::Event, parser::EventParser};
// use std::{
//     ffi::{CStr, CString},
//     fs::File,
//     io::BufReader,
//     path::PathBuf,
//     ptr,
// };
// use tracing::{debug, error, info, warn};
//
// mod convert;
// mod events;
// mod interruptor;
// mod types;
//
//
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     match do_main() {
//         Err(e) => {
//             error!("{}", e);
//             Err(e)
//         }
//         Ok(()) => Ok(()),
//     }
// }
//
// fn do_main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt::init();
//
//     let opts = Opts::parse();
//
//     let intr = Interruptor::new();
//     let intr_clone = intr.clone();
//     ctrlc::set_handler(move || {
//         if intr_clone.is_set() {
//             let exit_code = if cfg!(target_family = "unix") {
//                 // 128 (fatal error signal "n") + 2 (control-c is fatal error signal 2)
//                 130
//             } else {
//                 // Windows code 3221225786
//                 // -1073741510 == C000013A
//                 -1073741510
//             };
//             std::process::exit(exit_code);
//         }
//
//         debug!("Shutdown signal received");
//         intr_clone.set();
//     })?;
//
//     info!(input = %opts.input.display(), "Reading header info");
//     let file = File::open(&opts.input)?;
//     let reader = BufReader::new(file);
//
//     let output_path = CString::new(opts.output.to_str().unwrap())?;
//     let params = CtfPluginSinkFsInitParams::new(
//         Some(true), // assume_single_trace
//         None,       // ignore_discarded_events
//         None,       // ignore_discarded_packets
//         Some(true), // quiet
//         &output_path,
//     )?;
//
//     let state_inner: Box<dyn SourcePluginHandler> =
//         Box::new(TrcPluginState::new(intr, reader, &opts)?);
//     let state = Box::new(state_inner);
//
//     let mut pipeline = EncoderPipeline::new::<TrcPlugin>(opts.log_level, state, &params)?;
//
//     loop {
//         let run_status = pipeline.graph.run_once()?;
//         if RunStatus::End == run_status {
//             break;
//         }
//     }
//
//     info!("Done");
//
//     Ok(())
// }
