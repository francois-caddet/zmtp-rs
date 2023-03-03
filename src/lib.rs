pub mod errors;
pub use errors::Error;

pub type Result<T> = core::result::Result<T, errors::Error>;
