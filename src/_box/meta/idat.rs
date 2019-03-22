use crate::_box::BoxHeader;
use crate::bit::Stream;
use crate::Result;

#[derive(Debug, Default)]
pub struct ItemDataBox {
    pub box_header: BoxHeader,
    data: Vec<u8>,
}

impl ItemDataBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.box_header = box_header;
        s.parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        self.data = stream.read_bytes(stream.num_bytes_left())?.to_vec();
        Ok(self)
    }
}
