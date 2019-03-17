
use std::io::Result;

use crate::bbox::*;
use crate::bit::*;

#[derive(Debug)]
pub struct FileTypeBox {
    pub box_header: BoxHeader,
    pub major_brand: String,
    pub minor_version: String,
    pub compatibles: Vec<String>,
}

impl FileTypeBox {
    pub fn new(stream: &mut BitStream, box_header: BoxHeader) -> Result<FileTypeBox> {
        let mut size = box_header.box_size - u64::from(box_header.header_size());
        let major_brand = stream.read_4bytes()?.to_string();
        let minor_version = stream.read_4bytes()?.to_string();
        size -= 8;
        let mut compatibles = Vec::new();
        while size >= 4 {
            compatibles.push(stream.read_4bytes()?.to_string());
            size -= 4;
        }
        Ok(FileTypeBox {
            box_header,
            major_brand,
            minor_version,
            compatibles,
        })
    }
}