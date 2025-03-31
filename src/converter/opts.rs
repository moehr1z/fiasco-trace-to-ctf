use super::interruptor::Interruptor;
use super::plugin::{TrcPlugin, TrcPluginState};
use super::{convert::TrcCtfConverter, types::BorrowedCtfState};
use crate::parser::{event::Event, parser::EventParser};
use babeltrace2_sys::{
    BtResult, BtResultExt, CtfPluginSinkFsInitParams, EncoderPipeline, Error, LoggingLevel,
    MessageIteratorStatus, Plugin, RunStatus, SelfComponent, SelfMessageIterator,
    SourcePluginDescriptor, SourcePluginHandler, ffi, source_plugin_descriptors,
};
use chrono::prelude::{DateTime, Utc};
use clap::Parser;
use std::{
    ffi::{CStr, CString},
    fs::File,
    io::BufReader,
    path::PathBuf,
    ptr,
};
use tracing::{debug, error, info, warn};

/// Convert FreeRTOS trace-recorder traces to CTF
#[derive(Parser, Debug, Clone)]
#[clap(version)]
pub struct Opts {
    /// The CTF clock class name
    #[clap(long, default_value = "monotonic")]
    pub clock_name: String,

    /// The CTF trace name
    #[clap(long, default_value = "freertos")]
    pub trace_name: String,

    /// babeltrace2 log level
    #[clap(long, default_value = "warn")]
    pub log_level: LoggingLevel,

    /// Output directory to write traces to
    #[clap(short = 'o', long, default_value = "ctf_trace")]
    pub output: PathBuf,

    /// Path to the input trace recorder binary file (psf) to read
    pub input: PathBuf,
}
