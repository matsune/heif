use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct PrimaryItemBox {
    full_box_header: FullBoxHeader,
    item_id: u32,
}

impl Default for PrimaryItemBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new("pitm".parse().unwrap(), 0, 0),
            item_id: 0,
        }
    }
}

impl BBox for PrimaryItemBox {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PrimaryItemBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
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

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn item_id(&self) -> u32 {
        self.item_id
    }

    pub fn set_item_id(&mut self, id: u32) {
        self.item_id = id;
    }
}
