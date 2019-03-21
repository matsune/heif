use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
pub struct HandlerBox {
    pub full_box_header: FullBoxHeader,
    pub handler_type: String,
    pub name: String,
}

impl HandlerBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<HandlerBox> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        stream.skip_bytes(4)?;
        let handler_type = stream.read_4bytes()?.to_string();
        stream.skip_bytes(12)?;
        let mut name = Vec::new();
        while !stream.is_eof() {
            let c = stream.read_byte()?;
            if c == 0 {
                break;
            }
            name.push(c);
        }
        let name = String::from_utf8(name).unwrap();
        Ok(HandlerBox {
            full_box_header,
            handler_type,
            name,
        })
    }
}
