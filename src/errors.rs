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
    /// The remote ZMTP version is not compatible with the local version.
    /// As specified by ZMTP protocol, this could appen only when the remote is of a lower version.
    /// Right now, this crate does not provide any back compatibility mechanism.
    #[error("remote version incompatibility")]
    VersionMismatch(),
    /// The remote ZMTP mechanism is not compatible with the required one.
    /// Right now, this crate provide only NULL auth mechanism.
    #[error("remote authentification mechanism incompatibility")]
    MechanismMismatch(),
    /// Socket IO error.
    #[error("I/O {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ParseError {}
