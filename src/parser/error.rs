use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Given event type is unknown")]
    EventTypeError(u8),
    #[error("IO error on reading input")]
    Io(#[from] io::Error),
    #[error("Error on automatic parsing")]
    Parse(#[from] binrw::Error),
}
