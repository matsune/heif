use crate::bit::{Extract, Stream};
use crate::BoxHeader;
use crate::Result;

#[derive(Debug)]
pub struct FileTypeBox {
    pub box_header: BoxHeader,
    pub major_brand: String,
    pub minor_version: String,
    pub compatibles: Vec<String>,
}

impl FileTypeBox {
    pub fn new(extract: &mut Extract, box_header: BoxHeader) -> Result<FileTypeBox> {
        let major_brand = extract.read_4bytes()?.to_string();
        let minor_version = extract.read_4bytes()?.to_string();
        let mut compatibles = Vec::new();
        while !extract.is_eof() {
            compatibles.push(extract.read_4bytes()?.to_string());
        }
        Ok(FileTypeBox {
            box_header,
            major_brand,
            minor_version,
            compatibles,
        })
    }
}
