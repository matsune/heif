use crate::bbox::header::{BoxHeader, FullBoxHeader};
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

impl ItemDataBox {
    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        Ok(Self {
            box_header,
            data: stream.read_bytes(stream.num_bytes_left())?.to_vec(),
        })
    }
}
