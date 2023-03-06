//! ZMTP errors
use thiserror::Error;

/// These are the errors you may encounter using ZMTP.
#[derive(Error, Debug)]
pub enum Error {
    /// Network or socket connection errors
    #[error("Connection error, {0}")]
    Connection(#[from] ConnectionError),
    /// An error parsing a packet
    #[error("Parse error, {0}")]
    Parse(#[from] ParseError),
}

/// Internal connection error.
#[derive(Error, Debug)]
pub enum ConnectionError {
    /// Couldn't connect to the (host, port)
    #[error("cann't connect to tcp://{0}:{1}")]
    UnaccessibleHost(String, u16),
}

#[derive(Error, Debug)]
pub enum ParseError {}
