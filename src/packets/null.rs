use super::zmtp::{Flags, RawFrame};

#[derive(Debug)]
pub enum Command {
    Error(String),
    Ready {
        socket_type: Vec<u8>,
        identity: Option<Vec<u8>>,
    },
}

#[derive(Debug)]
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
                            for i in 0..4 {
                                raw_size[i] = tail[pos + i];
                            }
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
