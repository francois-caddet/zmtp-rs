//! Zmtp provided sockets (base, plain password, curve)
use crate::{errors::ConnectionError, Result};
use tokio::net::TcpStream;

/// The base ZMTP socket.
///
/// Does not provide any authentication/encryption security.
pub struct Zmtp(TcpStream);

impl Zmtp {
    /// Connect to `tcp://host:port`.
    ///
    /// Only provide tcp base transport for now. But will become generic over the base transport
    /// before `v0.0.2`
    ///
    /// # Exemple
    ///
    /// ```rust
    /// use zmtp::sockets;
    ///
    /// let port = 55555;
    /// let host = "localhost";
    /// sockets::Zmtp::connect(host, port);
    /// ```
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        TcpStream::connect((host, port))
            .await
            .map_err(|_| ConnectionError::UnaccessibleHost(host.to_string(), port).into())
            .map(Self)
    }
}
