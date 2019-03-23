use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct PrimaryItemBox {
    full_box_header: FullBoxHeader,
    item_id: u32,
}

impl PrimaryItemBox {
    pub fn new() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("pitm").unwrap(), 0, 0),
            item_id: 0,
        }
    }

    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from(stream, box_header)?;
        let item_id = if full_box_header.version() == 0 {
            stream.read_2bytes()?.to_u32()
        } else {
            stream.read_4bytes()?.to_u32()
        };
        Ok(Self {
            full_box_header,
            item_id,
        })
    }
}
