use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug, Default)]
pub struct PrimaryItemBox {
    pub full_box_header: FullBoxHeader,
    pub item_id: u32,
}

impl PrimaryItemBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.full_box_header = FullBoxHeader::new(stream, box_header)?;
        s.parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        self.item_id = if self.full_box_header.version == 0 {
            stream.read_2bytes()?.to_u32()
        } else {
            stream.read_4bytes()?.to_u32()
        };
        Ok(self)
    }
}
