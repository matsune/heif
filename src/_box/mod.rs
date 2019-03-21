pub mod ftyp;
pub mod meta;
pub mod moov;

use crate::bit::Stream;
use crate::Result;

pub trait Header {
    fn get_box_size(&self) -> u64;
    fn header_size(&self) -> u8;

    fn body_size(&self) -> u64 {
        self.get_box_size() - u64::from(self.header_size())
    }
}

#[derive(Debug)]
pub struct BoxHeader {
    pub box_size: u64,
    pub box_type: String,
    pub user_type: Vec<u8>,
    pub is_large: bool,
}

impl BoxHeader {
    pub fn new<T: Stream>(stream: &mut T) -> Result<BoxHeader> {
        let mut box_size = stream.read_4bytes()?.to_u64();
        let box_type = stream.read_4bytes()?.to_string();
        let mut is_large = false;
        let mut user_type = Vec::new();
        if box_size == 1 {
            box_size = stream.read_8bytes()?.to_u64();
            is_large = true;
        }
        if box_type == "uuid" {
            for _ in 0..16 {
                user_type.push(stream.read_byte()?)
            }
        }
        Ok(BoxHeader {
            box_size,
            box_type,
            is_large,
            user_type,
        })
    }
}

impl Header for BoxHeader {
    fn get_box_size(&self) -> u64 {
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
        let header = BoxHeader::new(&mut stream).unwrap();
        assert_eq!(header.box_size, 24);
        assert_eq!(header.box_type, "ftyp");
        assert_eq!(header.user_type, Vec::new());
        assert_eq!(header.is_large, false);
    }
}

#[derive(Debug)]
pub struct FullBoxHeader {
    pub box_size: u64,
    pub box_type: String,
    pub user_type: Vec<u8>,
    pub is_large: bool,
    pub version: u8,
    pub flags: u32,
}

impl FullBoxHeader {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let word = stream.read_4bytes()?;
        let version = word.0;
        let flags = u32::from(word.1) << 16 + u32::from(word.2) << 8 + u32::from(word.3);
        Ok(Self {
            box_size: box_header.box_size,
            box_type: box_header.box_type,
            user_type: box_header.user_type,
            is_large: box_header.is_large,
            version,
            flags,
        })
    }
}

impl Header for FullBoxHeader {
    fn get_box_size(&self) -> u64 {
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
