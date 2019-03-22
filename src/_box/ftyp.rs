use crate::Result;
use crate::_box::BoxHeader;
use crate::bit::Stream;

#[derive(Debug, Default)]
pub struct FileTypeBox {
    pub box_header: BoxHeader,
    pub major_brand: String,
    pub minor_version: String,
    pub compatibles: Vec<String>,
}

impl FileTypeBox {
    pub fn parse<T: Stream>(
        &mut self,
        extract: &mut T,
        box_header: BoxHeader,
    ) -> Result<&mut Self> {
        self.box_header = box_header;
        self.major_brand = extract.read_4bytes()?.to_string();
        self.minor_version = extract.read_4bytes()?.to_string();
        self.compatibles.clear();
        while !extract.is_eof() {
            self.compatibles.push(extract.read_4bytes()?.to_string());
        }
        Ok(self)
    }
}
