use crate::bit::{Byte4, Stream};
use crate::Result;
use std::str::FromStr;

pub trait Header {
    fn box_size(&self) -> u64;
    fn header_size(&self) -> u8;

    fn body_size(&self) -> u64 {
        self.box_size() - u64::from(self.header_size())
    }
}

#[derive(Debug, Default)]
pub struct BoxHeader {
    box_size: u64,
    box_type: Byte4,
    user_type: Vec<u8>,
    is_large: bool,
}

impl BoxHeader {
    pub fn new(box_type: Byte4) -> Self {
        Self {
            box_size: 8,
            box_type,
            user_type: Vec::new(),
            is_large: false,
        }
    }

    pub fn from<T: Stream>(stream: &mut T) -> Result<BoxHeader> {
        let mut box_size = stream.read_4bytes()?.to_u64();
        let box_type = stream.read_4bytes()?;
        let mut is_large = false;
        let mut user_type = Vec::new();
        if box_size == 1 {
            box_size = stream.read_8bytes()?.to_u64();
            is_large = true;
        }
        if box_type == "uuid" {
            user_type = stream.read_bytes(16)?.to_vec();
        }
        Ok(BoxHeader {
            box_size,
            box_type,
            is_large,
            user_type,
        })
    }

    pub fn box_type(&self) -> &Byte4 {
        &self.box_type
    }

    pub fn set_box_type(&mut self, b: Byte4) {
        self.box_type = b
    }

    pub fn set_box_size(&mut self, s: u64) {
        self.box_size = s
    }

    pub fn user_type(&self) -> &Vec<u8> {
        &self.user_type
    }

    pub fn set_user_type(&mut self, u: Vec<u8>) {
        self.set_box_type(Byte4::from_str("uuid").unwrap());
        self.user_type = u
    }

    pub fn is_large(&self) -> bool {
        self.is_large
    }

    pub fn set_large(&mut self) {
        self.is_large = true;
    }

    // TODO: update_size
}

impl Header for BoxHeader {
    fn box_size(&self) -> u64 {
        self.box_size
    }

    fn header_size(&self) -> u8 {
        let mut s = 8u8;
        if self.is_large {
            s += 8
        }
        if self.box_type == "uuid" {
            s += 16
        }
        s
    }
}

#[derive(Debug, Default)]
pub struct FullBoxHeader {
    box_size: u64,
    box_type: Byte4,
    user_type: Vec<u8>,
    is_large: bool,
    version: u8,
    flags: u32,
}

impl FullBoxHeader {
    pub fn new(box_type: Byte4, version: u8, flags: u32) -> Self {
        Self {
            box_size: 8,
            box_type,
            user_type: Vec::new(),
            is_large: false,
            version,
            flags,
        }
    }

    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let word = stream.read_4bytes()?;
        let version = word.0;
        let flags = (u32::from(word.1) << 16) + (u32::from(word.2) << 8) + u32::from(word.3);
        Ok(Self {
            box_size: box_header.box_size,
            box_type: box_header.box_type,
            user_type: box_header.user_type,
            is_large: box_header.is_large,
            version,
            flags,
        })
    }

    pub fn box_type(&self) -> &Byte4 {
        &self.box_type
    }

    pub fn set_box_type(&mut self, b: Byte4) {
        self.box_type = b
    }

    pub fn set_box_size(&mut self, s: u64) {
        self.box_size = s
    }

    pub fn user_type(&self) -> &Vec<u8> {
        &self.user_type
    }

    pub fn set_user_type(&mut self, u: Vec<u8>) {
        self.set_box_type(Byte4::from_str("uuid").unwrap());
        self.user_type = u
    }

    pub fn is_large(&self) -> bool {
        self.is_large
    }

    pub fn set_large(&mut self) {
        self.is_large = true;
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn set_version(&mut self, ver: u8) {
        self.version = ver
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags
    }
}

impl Header for FullBoxHeader {
    fn box_size(&self) -> u64 {
        self.box_size
    }

    fn header_size(&self) -> u8 {
        let mut s = 8u8;
        if self.is_large {
            s += 8
        }
        if self.box_type == "uuid" {
            s += 16
        }
        s + 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bit::BitStream;

    #[test]
    fn test_box_header_new() {
        // 00 00 00 18  24
        // 66 74 79 70  ftyp
        // 6D 69 66 31  mif1
        // 00 00 00 00
        // 6D 69 66 31  mif1
        // 68 65 69 63  heic
        let mut stream = BitStream::new(vec![
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x6D, 0x69, 0x66, 0x31, 0x00, 0x00,
            0x00, 0x00, 0x6D, 0x69, 0x66, 0x31, 0x68, 0x65, 0x69, 0x63,
        ]);
        let header = BoxHeader::from(&mut stream).unwrap();
        assert_eq!(header.box_size, 24);
        assert_eq!(header.box_type, "ftyp");
        assert_eq!(header.user_type, Vec::new());
        assert_eq!(header.is_large, false);
    }
}
