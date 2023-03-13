pub mod errors;
pub use errors::Error;

pub mod packets;
pub mod sockets;

/// Returned by every ZMTP's function which may fail.
pub type Result<T> = core::result::Result<T, errors::Error>;
