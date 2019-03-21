use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
pub struct ItemReferenceBox {
    pub full_box_header: FullBoxHeader,
    pub reference_list: Vec<SingleItemTypeReferenceBox>,
}

impl ItemReferenceBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let is_large = full_box_header.version > 0;
        let mut reference_list = Vec::new();
        while !stream.is_eof() {
            let box_header = BoxHeader::new(stream)?;
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
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader, is_large: bool) -> Result<Self> {
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
