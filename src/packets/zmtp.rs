use bytes::Bytes;

#[repr(C, packed)]
#[derive(Debug)]
pub struct Greeting {
    signature: [u8; 10],
    version: Version,
    mechanism: Mechanism,
    as_server: u8,
    filler: [u8; 31],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
}

const fn zerro_padded<const M: usize, const N: usize>(arr: &[u8; M]) -> [u8; N] {
    let mut ret = [0u8; N];
    let mut i = 0;
    while i < M {
        ret[i] = arr[i];
        i += 1;
    }
    ret
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mechanism(pub [u8; 20]);

impl Mechanism {
    pub const NULL: Self = Self(zerro_padded(br#"NULL"#));
}

#[repr(C, packed)]
pub struct FrameType<S: FrameSize> {
    pub flags: Flags,
    pub size: S,
}

impl<T> FrameType<T>
where
    T: FrameSize + Copy,
{
    pub async fn with_stream<S: futures::Stream<Item = Result<Bytes, std::io::Error>>>(
        self,
        bytes: S,
    ) -> RawFrame {
        use futures::{pin_mut, StreamExt};
        pin_mut!(bytes);
        let mut buf: Vec<u8> = Vec::new();
        while buf.len() < self.size.into() as usize {
            if let Some(chunk) = bytes.next().await {
                buf.extend(chunk.unwrap());
            } else {
                break;
            }
        }
        if self.flags.is_command() {
            RawFrame::Command(buf)
        } else {
            RawFrame::Message(buf)
        }
    }
}

pub trait FrameSize: Into<u64> + Sized {}

impl FrameSize for u8 {}
impl FrameSize for u64 {}

pub enum RawFrame {
    Command(Vec<u8>),
    Message(Vec<u8>),
}

#[repr(C, packed)]
#[derive(Debug, Default)]
pub struct Flags(pub u8);

impl Flags {
    const TYPE_MASK: Self = Self(0b00000100);
    const SIZE_MASK: Self = Self(0b00000010);
    const LAST: Self = Self(0b00000001);

    pub fn command(self) -> Self {
        // set bit 2 to 1 and keep the 2 others value
        Self(self.0 | 0b00000100)
    }

    pub fn message(self) -> Self {
        // set bit 2 to 0 and keep the 2 others value
        Self(self.0 & 0b00000011)
    }

    pub fn big(self) -> Self {
        // set bit 1 to 1 and keep the 2 others value
        Self(self.0 | 0b00000010)
    }

    pub fn small(self) -> Self {
        // set bit 1 to 0 and keep the 2 others value
        Self(self.0 & 0b00000101)
    }

    pub fn more(self) -> Self {
        // set bit 0 to 1 and keep the 2 others value
        Self(self.0 | 0b00000001)
    }

    pub fn last(self) -> Self {
        // set bit 0 to 0 and keep the 2 others value
        Self(self.0 & 0b00000110)
    }

    pub fn is_command(&self) -> bool {
        self.0 & Self::TYPE_MASK.0 > 0
    }

    pub fn is_message(&self) -> bool {
        self.0 & Self::TYPE_MASK.0 == 0
    }

    pub fn is_big(&self) -> bool {
        self.0 & Self::SIZE_MASK.0 > 0
    }

    pub fn is_small(&self) -> bool {
        self.0 & Self::SIZE_MASK.0 == 0
    }

    pub fn is_more(&self) -> bool {
        self.0 & Self::LAST.0 > 0
    }

    pub fn is_last(&self) -> bool {
        self.0 & Self::LAST.0 == 0
    }
}

impl Greeting {
    const SIG: [u8; 10] = [0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0x7f];
    const FILLER: [u8; 31] = [0x00; 31];

    pub fn new() -> Self {
        Self {
            signature: Self::SIG,
            version: Version { major: 3, minor: 0 },
            mechanism: Mechanism::NULL,
            as_server: 0u8,
            filler: Self::FILLER,
        }
    }

    pub fn with_mechanism(mut self, mechanism: Mechanism) -> Self {
        self.mechanism = mechanism;
        self
    }
}

impl Default for Greeting {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) trait Packet: Sized {
    fn as_bytes(&self) -> &[u8];
    fn from_bytes(buf: &[u8]) -> &Self;
}

impl Packet for Greeting {
    fn as_bytes(&self) -> &[u8] {
        unsafe { ::core::slice::from_raw_parts((self as *const Self) as *const u8, 64) }
    }

    fn from_bytes(buf: &[u8]) -> &Self {
        let (head, body, _tail) = unsafe { buf.align_to::<Self>() };
        assert!(head.is_empty(), "Data was not aligned");
        &body[0]
    }
}

impl<T: FrameSize> Packet for FrameType<T> {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            ::core::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }

    fn from_bytes(buf: &[u8]) -> &Self {
        let (head, body, _tail) = unsafe { buf.align_to::<Self>() };
        assert!(head.is_empty(), "Data was not aligned");
        &body[0]
    }
}
