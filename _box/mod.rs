pub mod ftyp;
//pub mod meta;
//
use crate::bit::*;
use crate::Result;

#[derive(Debug)]
pub struct BoxHeader {
    pub box_size: u64,
    pub box_type: String,
    pub user_type: Vec<u8>,
    pub is_large: bool,
}

impl BoxHeader {
    //  pub fn header_size(&self) -> u8 {
    //      let mut s = 8u8;
    //      if self.box_size == 1 {
    //          s += 8
    //      }
    //      if self.box_type == "uuid" {
    //          s += 16
    //      }
    //      s
    //  }

    pub fn new(stream: &mut BitStream) -> Result<BoxHeader> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    pub flags: (u8, u8, u8),
}

impl FullBoxHeader {
    pub fn new(stream: &mut BitStream, box_header: BoxHeader) -> Result<Self> {
        let word = stream.read_4bytes()?;
        let version = word.0;
        let flags = (word.1, word.2, word.3);
        Ok(Self {
            box_size: box_header.box_size,
            box_type: box_header.box_type,
            user_type: box_header.user_type,
            is_large: box_header.is_large,
            version,
            flags,
        })
    }

    //    pub fn header_size(&self) -> u8 {
    //        self.box_header.header_size() + 4
    //    }
    //
    //    pub fn box_size(&self) -> u64 {
    //        self.box_header.box_size
    //    }
}
