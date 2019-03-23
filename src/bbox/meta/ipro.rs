use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct ItemProtectionBox {
    full_box_header: FullBoxHeader,
    protection_info: Vec<ProtectionSchemeInfoBox>,
}

impl ItemProtectionBox {
    pub fn new() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("ipro").unwrap(), 0, 0),
            protection_info: Vec::new(),
        }
    }

    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from(stream, box_header)?;
        let mut protection_info = Vec::new();
        let box_count = stream.read_2bytes()?.to_u16();
        for _ in 0..box_count {
            let child_box_header = BoxHeader::from(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            protection_info.push(ProtectionSchemeInfoBox::from(&mut ex)?);
        }
        Ok(Self {
            full_box_header,
            protection_info,
        })
    }
}

#[derive(Debug)]
pub struct ProtectionSchemeInfoBox {
    data: Vec<u8>,
}

impl ProtectionSchemeInfoBox {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn from<T: Stream>(stream: &mut T) -> Result<Self> {
        Ok(Self {
            data: stream.read_bytes(stream.num_bytes_left())?.to_vec(),
        })
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}