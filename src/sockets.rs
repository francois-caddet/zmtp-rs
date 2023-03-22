//! Zmtp provided sockets (base, plain password, curve)
use crate::packets::null;
use crate::Result;

use futures::{StreamExt, TryFutureExt};

/// The base ZMTP socket.
///
/// Does not provide any authentication/encryption security.
pub struct Zmtp(states::FrameStream);

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
        states::Root::connect(host, port)
            .and_then(|c| c.version(3, 0))
            .and_then(|c| c.mechanism(crate::packets::Mechanism::NULL))
            .map_ok(|c| Self(c.frame_stream()))
            .err_into()
            .await
    }

    pub fn version(&self) -> crate::packets::Version {
        crate::packets::Version { major: 3, minor: 0 }
    }

    pub async fn next_frame(&mut self) -> Option<null::Frame> {
        self.0.next().await
    }

    pub async fn send_frame(&mut self, frame: null::Frame) -> crate::Result<()> {
        self.0.send(frame).err_into().await
    }
}

mod states {
    use crate::errors::ConnectionError;
    use crate::packets::{null, Flags, Greeting, Packet};

    use futures::{Stream, TryFutureExt};
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    pub struct Root;
    impl Root {
        pub async fn connect(host: &str, port: u16) -> Result<Connected, ConnectionError> {
            TcpStream::connect((host, port))
                .map_ok(Connected)
                .map_err(|_| ConnectionError::UnaccessibleHost(host.to_string(), port))
                .await
        }
    }

    pub struct Connected(TcpStream);
    impl Connected {
        pub async fn version(self, major: u8, minor: u8) -> Result<Versioned, ConnectionError> {
            if (major, minor) != (3u8, 0u8) {
                return Err(ConnectionError::VersionMismatch());
            }
            let greeting = Greeting::new();
            let (mut reader, mut writer) = split(self.0);
            tokio::try_join![
                async {
                    let mut buf = [0u8; 11];
                    reader.read_exact(&mut buf).await?;
                    match buf {
                        [0xff, _, _, _, _, _, _, _, _, last, v] if (last & 0x01 > 0) && v >= 3 => {
                            Ok(())
                        }
                        _ => Err(ConnectionError::VersionMismatch()),
                    }
                },
                writer.write_all(&greeting.as_bytes()[..11]).err_into(),
            ]?;
            Ok(Versioned(reader.unsplit(writer), greeting))
        }
    }

    pub struct Versioned(TcpStream, Greeting);
    impl Versioned {
        pub async fn mechanism(
            self,
            m: crate::packets::Mechanism,
        ) -> Result<AgreedMechanism, ConnectionError> {
            use crate::packets::Mechanism;
            let (mut reader, mut writer) = split(self.0);
            tokio::try_join![
                async {
                    let mut remote_m = [0u8; 20];
                    // ignore the 12th byte. it represent the minor version in ZMTP 3.0
                    reader.read_exact(&mut [0u8; 1]).await?;
                    reader.read_exact(&mut remote_m).await?;
                    if m == Mechanism(remote_m) {
                        reader
                            .read_exact(&mut [0u8; 32])
                            .map_ok(|_| ())
                            .err_into()
                            .await
                    } else {
                        Err(ConnectionError::MechanismMismatch())
                    }
                },
                writer.write_all(&self.1.as_bytes()[11..]).err_into(),
            ]?;
            Ok(AgreedMechanism(reader.unsplit(writer)))
        }
    }

    use tokio_util::io::ReaderStream;
    pub struct AgreedMechanism(TcpStream);
    impl AgreedMechanism {
        pub fn frame_stream(self) -> FrameStream {
            FrameStream(self.0, false)
        }
    }

    pub struct FrameStream(TcpStream, bool);
    impl FrameStream {
        pub async fn send(
            &mut self,
            frame: null::Frame,
        ) -> Result<(), crate::errors::ConnectionError> {
            self.0.write_all(&frame.to_vec_u8()).await?;
            self.0.flush().err_into().await
        }
    }
    impl Stream for FrameStream {
        type Item = null::Frame;
        fn poll_next(
            self: core::pin::Pin<&mut Self>,
            cx: &mut futures::task::Context,
        ) -> futures::task::Poll<Option<Self::Item>> {
            use crate::packets::FrameType;
            use futures::Future;
            let s = async {
                //    if self.1 {
                //        return None;
                //    }
                let mut_self = self.get_mut();
                let flags = Flags(mut_self.0.read_u8().await.unwrap());
                println!("recv flags: {:02X?}", flags);
                //    mut_self.1 = flags.is_last();
                let raw_frame = if flags.is_big() {
                    let size = mut_self.0.read_u64().await.unwrap();
                    FrameType { flags, size }
                        .with_stream(ReaderStream::new(&mut mut_self.0))
                        .await
                } else {
                    let size = mut_self.0.read_u8().await.unwrap();
                    FrameType { flags, size }
                        .with_stream(ReaderStream::new(&mut mut_self.0))
                        .await
                };
                Some(raw_frame.try_into().unwrap())
            };
            futures::pin_mut!(s);
            s.poll(cx)
        }
    }
}
