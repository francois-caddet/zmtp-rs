pub mod errors;
pub use errors::Error;

/// Returned by every ZMTP's function which may fail.
pub type Result<T> = core::result::Result<T, errors::Error>;
