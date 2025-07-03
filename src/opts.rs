use babeltrace2_sys::LoggingLevel;
use clap::Parser;
use std::path::PathBuf;

/// Convert L4Re traces to CTF
#[derive(Parser, Debug, Clone)]
#[clap(version)]
pub struct Opts {
    /// The CTF clock class name
    #[clap(long, default_value = "monotonic")]
    pub clock_name: String,

    /// The clock frequency
    #[clap(short = 'f', long)]
    pub clock_frequency: u64,

    /// The CTF trace name
    #[clap(long, default_value = "l4re")]
    pub trace_name: String,

    /// babeltrace2 log level
    #[clap(long, default_value = "warn")]
    pub log_level: LoggingLevel,

    /// Output directory to write traces to
    #[clap(short = 'o', long, default_value = "ctf_trace")]
    pub output: PathBuf,
}
