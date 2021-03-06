use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

#[derive(Debug)]
pub struct HandlerBox {
    full_box_header: FullBoxHeader,
    handler_type: Byte4,
    name: String,
}

impl BBox for HandlerBox {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Default for HandlerBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new("hdlr".parse().unwrap(), 0, 0),
            handler_type: Byte4::default(),
            name: String::new(),
        }
    }
}

impl HandlerBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
        stream.skip_bytes(4)?;
        let handler_type = stream.read_4bytes()?;
        stream.skip_bytes(12)?;
        let name = stream.read_zero_term_string();
        Ok(Self {
            full_box_header,
            handler_type,
            name,
        })
    }

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn handler_type(&self) -> &Byte4 {
        &self.handler_type
    }

    pub fn set_handler_type(&mut self, handler_type: Byte4) {
        self.handler_type = handler_type
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }
}
