use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct ItemReferenceBox {
    full_box_header: FullBoxHeader,
    reference_list: Vec<SingleItemTypeReferenceBox>,
}

impl ItemReferenceBox {
    pub fn new() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("iref").unwrap(), 0, 0),
            reference_list: Vec::new(),
        }
    }

    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from(stream, box_header)?;
        let is_large = full_box_header.version() > 0;
        let mut reference_list = Vec::new();
        while !stream.is_eof() {
            let box_header = BoxHeader::from(stream)?;
            reference_list.push(SingleItemTypeReferenceBox::from(
                stream, box_header, is_large,
            )?);
        }
        Ok(Self {
            full_box_header,
            reference_list,
        })
    }
}

#[derive(Debug)]
pub struct SingleItemTypeReferenceBox {
    box_header: BoxHeader,
    from_item_id: u32,
    to_item_ids: Vec<u32>,
    is_large: bool,
}

impl SingleItemTypeReferenceBox {
    pub fn new(is_large: bool) -> Self {
        Self {
            box_header: BoxHeader::new(Byte4::default()),
            from_item_id: 0,
            to_item_ids: Vec::new(),
            is_large,
        }
    }

    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader, is_large: bool) -> Result<Self> {
        let from_item_id = if is_large {
            stream.read_4bytes()?.to_u32()
        } else {
            stream.read_2bytes()?.to_u32()
        };
        let ref_count = stream.read_2bytes()?.to_u16();
        let mut to_item_ids = Vec::new();
        for _ in 0..ref_count {
            to_item_ids.push(if is_large {
                stream.read_4bytes()?.to_u32()
            } else {
                stream.read_2bytes()?.to_u32()
            })
        }
        Ok(Self {
            box_header,
            from_item_id,
            to_item_ids,
            is_large,
        })
    }
}
