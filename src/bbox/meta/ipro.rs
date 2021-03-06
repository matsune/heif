use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

#[derive(Debug)]
pub struct ItemProtectionBox {
    full_box_header: FullBoxHeader,
    protection_info: Vec<ProtectionSchemeInfoBox>,
}

impl Default for ItemProtectionBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new("ipro".parse().unwrap(), 0, 0),
            protection_info: Vec::new(),
        }
    }
}

impl BBox for ItemProtectionBox {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemProtectionBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
        let mut protection_info = Vec::new();
        let box_count = stream.read_2bytes()?.to_u16();
        for _ in 0..box_count {
            let child_box_header = BoxHeader::from_stream(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            protection_info.push(ProtectionSchemeInfoBox::from_stream(&mut ex)?);
        }
        Ok(Self {
            full_box_header,
            protection_info,
        })
    }

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn protection_count(&self) -> usize {
        self.protection_info.len()
    }

    pub fn entry_at(&self, index: usize) -> Option<&ProtectionSchemeInfoBox> {
        self.protection_info.get(index)
    }

    pub fn add_entry(&mut self, entry: ProtectionSchemeInfoBox) {
        self.protection_info.push(entry)
    }
}

#[derive(Debug)]
pub struct ProtectionSchemeInfoBox {
    data: Vec<u8>,
}

impl Default for ProtectionSchemeInfoBox {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl ProtectionSchemeInfoBox {
    pub fn from_stream<T: Stream>(stream: &mut T) -> Result<Self> {
        Ok(Self {
            data: stream.read_bytes(stream.num_bytes_left())?.to_vec(),
        })
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}
