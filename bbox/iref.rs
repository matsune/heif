use crate::bbox::{BoxHeader, FullBoxHeader};
use crate::bit::BitStream;
use std::io::Result;

#[derive(Debug)]
pub struct ItemReferenceBox {
    pub full_box_header: FullBoxHeader,
    pub reference_list: Vec<SingleItemTypeReferenceBox>,
}

impl ItemReferenceBox {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<Self> {
        let is_large = full_box_header.version > 0;
        let mut reference_list = Vec::new();
        let mut left = full_box_header.box_size() - u64::from(full_box_header.header_size());
        while left > 0 {
            let box_header = BoxHeader::new(stream)?;
            left -= box_header.box_size;
            reference_list.push(SingleItemTypeReferenceBox::new(
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
    pub box_header: BoxHeader,
    pub from_item_id: u32,
    pub to_item_ids: Vec<u32>,
    pub is_large: bool,
}

impl SingleItemTypeReferenceBox {
    pub fn new(stream: &mut BitStream, box_header: BoxHeader, is_large: bool) -> Result<Self> {
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
