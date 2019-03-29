use crate::bbox::header::BoxHeader;
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

#[derive(Debug)]
pub struct FileTypeBox {
    pub box_header: BoxHeader,
    pub major_brand: Byte4,
    pub minor_version: u32,
    pub compatibles: Vec<String>,
}

impl std::default::Default for FileTypeBox {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new("ftyp".parse().unwrap()),
            major_brand: Byte4::default(),
            minor_version: 0,
            compatibles: Vec::new(),
        }
    }
}

impl BBox for FileTypeBox {
    fn box_type(&self) -> &Byte4 {
        self.box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FileTypeBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let box_header = box_header;
        let major_brand = stream.read_4bytes()?;
        let minor_version = stream.read_4bytes()?.to_u32();
        let mut compatibles = Vec::new();
        while !stream.is_eof() {
            compatibles.push(stream.read_4bytes()?.to_string());
        }
        Ok(Self {
            box_header,
            major_brand,
            minor_version,
            compatibles,
        })
    }
}
