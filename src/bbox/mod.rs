pub mod ftyp;
pub mod meta;

use std::io::Result;
use crate::bit::*;

#[derive(Debug)]
pub struct BoxHeader {
    pub box_size: u64,
    pub box_type: String,
    pub user_type: Vec<u8>,
    pub is_large: bool,
}

impl BoxHeader {
    pub fn header_size(&self) -> u8 {
        let mut s = 8u8;
        if self.box_size == 1 {
            s += 8
        }
        if self.box_type == "uuid" {
            s += 16
        }
        s
    }

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

#[derive(Debug)]
pub struct FullBoxHeader {
    pub box_header: BoxHeader,
    pub version: u8,
    pub flags: (u8, u8, u8),
}

impl FullBoxHeader {
    pub fn new(stream: &mut BitStream, box_header: BoxHeader) -> Result<FullBoxHeader> {
        let word = stream.read_4bytes()?;
        let version = word.0;
        let flags = (word.1, word.2, word.3);
        Ok(FullBoxHeader {
            box_header,
            version,
            flags,
        })
    }

    pub fn header_size(&self) -> u8 {
        self.box_header.header_size() + 4
    }

    pub fn box_size(&self) -> u64 {
        self.box_header.box_size
    }
}
