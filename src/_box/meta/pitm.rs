use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
pub struct PrimaryItemBox {
    pub full_box_header: FullBoxHeader,
    pub item_id: u32,
}

impl PrimaryItemBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<PrimaryItemBox> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
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
