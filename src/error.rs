use thiserror::Error;
use abi_stable::StableAbi;
use abi_stable::std_types::{RString, RIoError};

#[repr(u8)]
#[derive(Error,Debug,StableAbi)]
pub enum Error {
    #[error("IO error")]
    IoError(#[from] RIoError),

    #[error("Error in plugin: {0:}")]
    Plugin(RString),

    #[error("Signal not found: '{0:}'")]
    NotFound(RString),
}


