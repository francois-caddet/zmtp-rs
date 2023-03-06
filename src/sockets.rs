use crate::{errors::ConnectionError, Result};
use tokio::net::TcpStream;

pub struct Zmtp(TcpStream);

impl Zmtp {
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        TcpStream::connect((host, port))
            .await
            .map_err(|_| ConnectionError::UnaccessibleHost(host.to_string(), port).into())
            .map(Self)
    }
}
