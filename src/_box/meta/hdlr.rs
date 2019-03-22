use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug, Default)]
pub struct HandlerBox {
    pub full_box_header: FullBoxHeader,
    pub handler_type: String,
    pub name: String,
}

impl HandlerBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.full_box_header = FullBoxHeader::new(stream, box_header)?;
        s.parse(stream)
    }

    pub fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        stream.skip_bytes(4)?;
        self.handler_type = stream.read_4bytes()?.to_string();
        stream.skip_bytes(12)?;
        let mut name = Vec::new();
        while !stream.is_eof() {
            let c = stream.read_byte()?;
            if c == 0 {
                break;
            }
            name.push(c);
        }
        self.name = String::from_utf8(name).unwrap();
        Ok(self)
    }
}
