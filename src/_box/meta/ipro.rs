use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
pub struct ItemProtectionBox {
    pub full_box_header: FullBoxHeader,
    pub protection_info: Vec<ProtectionSchemeInfoBox>,
}

impl ItemProtectionBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let mut protection_info = Vec::new();
        let box_count = stream.read_2bytes()?.to_u16();
        for _ in 0..box_count {
            let child_box_header = BoxHeader::new(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            protection_info.push(ProtectionSchemeInfoBox::new(&mut ex)?);
        }
        Ok(Self {
            full_box_header,
            protection_info,
        })
    }
}

#[derive(Debug)]
pub struct ProtectionSchemeInfoBox {
    data: Vec<u8>,
}

impl ProtectionSchemeInfoBox {
    pub fn new<T: Stream>(stream: &mut T) -> Result<Self> {
        Ok(Self {
            data: stream.read_bytes(stream.num_bytes_left())?.to_vec(),
        })
    }
}