use crate::_box::BoxHeader;
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
pub struct ItemDataBox {
    pub box_header: BoxHeader,
    data: Vec<u8>,
}

impl ItemDataBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let data = stream.read_bytes(stream.num_bytes_left())?.to_vec();
        Ok(Self { box_header, data })
    }
}
