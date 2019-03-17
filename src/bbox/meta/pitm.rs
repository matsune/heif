use crate::bbox::FullBoxHeader;
use crate::bit::BitStream;
use std::io::Result;

#[derive(Debug)]
pub struct PrimaryItemBox {
    pub full_box_header: FullBoxHeader,
    pub item_id: u32,
}

impl PrimaryItemBox {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<PrimaryItemBox> {
        let item_id = if full_box_header.version == 0 {
            stream.read_2bytes()?.to_u32()
        } else {
            stream.read_4bytes()?.to_u32()
        };
        Ok(PrimaryItemBox {
            full_box_header,
            item_id,
        })
    }
}
