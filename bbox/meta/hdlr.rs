use crate::bbox::FullBoxHeader;
use crate::bit::BitStream;
use std::io::Result;

#[derive(Debug)]
pub struct HandlerBox {
    pub full_box_header: FullBoxHeader,
    pub handler_type: String,
    pub name: String,
}

impl HandlerBox {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<HandlerBox> {
        stream.skip_bytes(4)?;
        let handler_type = stream.read_4bytes()?.to_string();
        stream.skip_bytes(12)?;
        let size = full_box_header.box_size() - u64::from(full_box_header.header_size());
        let mut name = Vec::new();
        for _ in 0..(size - 20) {
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
