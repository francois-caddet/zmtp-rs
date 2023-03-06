#[repr(C)]
pub struct Greeting {
    signature: [u8; 10],
    version: Version,
    mechanism: [char; 20],
    as_server: u8,
    filler: [u8; 31],
}

#[repr(C)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
}

impl Greeting {
    const SIG: [u8; 10] = [0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0xf7];
    const FILLER: [u8; 31] = [0x00; 31];

    pub fn new() -> Self {
        Self {
            signature: Self::SIG,
            version: Version { major: 3, minor: 0 },
            mechanism: [0 as char; 20],
            as_server: 0u8,
            filler: Self::FILLER,
        }
    }
}
