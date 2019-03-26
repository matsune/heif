use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct ItemDataBox {
    box_header: BoxHeader,
    data: Vec<u8>,
}

impl Default for ItemDataBox {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new(Byte4::from_str("idat").unwrap()),
            data: Vec::new(),
        }
    }
}

impl BBox for ItemDataBox {
    type HeaderType = BoxHeader;
    fn header(&self) -> &Self::HeaderType {
        &self.box_header
    }
}

impl ItemDataBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        Ok(Self {
            box_header,
            data: stream.read_bytes(stream.num_bytes_left())?.to_vec(),
        })
    }

    pub fn box_header(&self) -> &BoxHeader {
        &self.box_header
    }

    pub fn read(&self, offset: usize, length: usize) -> Option<&[u8]> {
        if (offset + length) > self.data.len() {
            return None;
        }
        Some(&self.data[offset..(offset + length)])
    }

    pub fn add_data(&mut self, data: &mut Vec<u8>) {
        self.data.append(data);
    }
}
