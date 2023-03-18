use super::zmtp::RawFrame;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Error(String),
    Ready {
        socket_type: Vec<u8>,
        identity: Option<Vec<u8>>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    Command(Command),
}

impl TryFrom<RawFrame> for Frame {
    type Error = ();
    fn try_from(f: RawFrame) -> Result<Frame, ()> {
        Ok(match f {
            RawFrame::Command(ref arr) => {
                let size = arr[0] as usize;
                let tail = &arr[1..];
                let command = match &tail[..size] {
                    br#"ERROR"# => {
                        Command::Error(String::from_utf8(tail[size + 1..].into()).unwrap())
                    }
                    br#"READY"# => {
                        let mut socket_type = None;
                        let mut identity = None;
                        let mut pos = size;
                        while pos < tail.len() {
                            let size = tail[pos] as usize;
                            pos += 1;
                            let key = &tail[pos..pos + size];
                            pos += size;
                            let mut raw_size: [u8; 4] = [0u8; 4];
                            raw_size[..4].copy_from_slice(&tail[pos..pos + 4]);
                            let size = u32::from_be_bytes(raw_size) as usize;
                            println!("{}", size);
                            pos += 4;
                            let val = Vec::from(&tail[pos..pos + size]);
                            pos += size;
                            println!("{}", String::from_utf8(key.into()).unwrap());
                            match key {
                                br#"Socket-Type"# => socket_type = Some(val),
                                br#"Identity"# => identity = Some(val),
                                _ => (),
                            }
                        }
                        let socket_type = socket_type.unwrap();
                        Command::Ready {
                            socket_type,
                            identity,
                        }
                    }
                    cmd => panic!(
                        "{} not supported by this protocol",
                        String::from_utf8(cmd.into()).unwrap()
                    ),
                };
                Frame::Command(command)
            }
            RawFrame::Message(_) => todo!(),
        })
    }
}

impl Command {
    pub fn to_vec_u8(&self) -> Vec<u8> {
        match self {
            Command::Ready {
                socket_type,
                identity,
            } => {
                let key = br#"READY"#;
                let mut buf = Vec::new();
                buf.push(key.len() as u8);
                buf.extend(key);
                let key = br#"Socket-Type"#;
                buf.push(key.len() as u8);
                buf.extend(key);
                buf.extend(&(socket_type.len() as u32).to_be_bytes());
                buf.extend(socket_type);
                if let Some(identity) = identity {
                    let key = br#"Identity"#;
                    buf.push(key.len() as u8);
                    buf.extend(key);
                    buf.extend(&(identity.len() as u32).to_be_bytes());
                    buf.extend(identity);
                }
                buf
            }
            Command::Error(_msg) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Frame};
    use crate::packets;

    #[test]
    fn simetric_serialize() {
        let cmd = Command::Ready {
            socket_type: Vec::from(&b"REQ"[..]),
            identity: Some(Vec::from(&b"test.identity"[..])),
        };
        assert_eq!(
            Frame::try_from(packets::RawFrame::Command(cmd.to_vec_u8())),
            Ok(Frame::Command(cmd))
        );
    }
}
