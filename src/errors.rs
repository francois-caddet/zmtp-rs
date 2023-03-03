use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error, {0}")]
    Connection(#[from] ConnectionError),
    #[error("Parse error, {0}")]
    Parse(#[from] ParseError),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("cann't connect to tcp://{0}:{1}")]
    UnaccessibleHost(String, u16),
}

#[derive(Error, Debug)]
pub enum ParseError {}
