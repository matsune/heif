use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::{HeifError, Result};

use std::str::FromStr;

#[derive(Debug)]
pub struct DataInformationBox {
    box_header: BoxHeader,
    data_reference_box: DataReferenceBox,
}

impl Default for DataInformationBox {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new(Byte4::from_str("dinf").unwrap()),
            data_reference_box: DataReferenceBox::default(),
        }
    }
}

impl BBox for DataInformationBox {
    fn box_type(&self) -> &Byte4 {
        self.box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DataInformationBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let data_reference_box = if stream.is_eof() {
            DataReferenceBox::default()
        } else {
            let mut ex = stream.extract_from(&box_header)?;
            let child_box_header = BoxHeader::from_stream(&mut ex)?;
            DataReferenceBox::from_stream_header(&mut ex, child_box_header)?
        };
        Ok(Self {
            box_header,
            data_reference_box,
        })
    }

    pub fn box_header(&self) -> &BoxHeader {
        &self.box_header
    }

    pub fn add_entry(&mut self, entry: DataEntryBox) {
        self.data_reference_box.add_entry(entry);
    }
}

pub struct DataReferenceBox {
    full_box_header: FullBoxHeader,
    data_entries: Vec<DataEntryBox>,
}

impl std::fmt::Debug for DataReferenceBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DataReferenceBox {:?}", self.full_box_header)
    }
}

impl BBox for DataReferenceBox {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Default for DataReferenceBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("dref").unwrap(), 0, 0),
            data_entries: Vec::new(),
        }
    }
}

impl DataReferenceBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        Self {
            full_box_header: FullBoxHeader::from_stream_header(stream, box_header)?,
            data_entries: Vec::new(),
        }
        .parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        let entry_count = stream.read_4bytes()?.to_u32();
        self.data_entries.clear();
        for _ in 0..entry_count {
            let child_box_header = BoxHeader::from_stream(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            let data_entry = match child_box_header.box_type().to_string().as_str() {
                "urn " => DataEntryBox::from_stream_header_urn(&mut ex, child_box_header)?,
                "url " => DataEntryBox::from_stream_header_url(&mut ex, child_box_header)?,
                _ => return Err(HeifError::Unknown("An unknown box inside dref")),
            };
            self.data_entries.push(data_entry);
        }
        Ok(self)
    }

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn add_entry(&mut self, entry: DataEntryBox) {
        self.data_entries.push(entry);
    }
}

#[derive(Debug)]
pub struct DataEntryBox {
    full_box_header: FullBoxHeader,
    location: String,
    name: String, // used urn only
}

impl BBox for DataEntryBox {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DataEntryBox {
    pub fn new(box_type: Byte4, version: u8, flags: u32) -> Self {
        Self {
            full_box_header: FullBoxHeader::new(box_type, version, flags),
            location: String::new(),
            name: String::new(),
        }
    }

    pub fn new_urn() -> Self {
        Self::new(Byte4::from_str("urn ").unwrap(), 0, 0)
    }

    pub fn new_url(is_self_contained: bool) -> Self {
        Self::new(Byte4::from_str("url ").unwrap(), 0, 0)
    }

    pub fn from_stream_header_urn<T: Stream>(
        stream: &mut T,
        box_header: BoxHeader,
    ) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
        let name = stream.read_zero_term_string();
        let location = stream.read_zero_term_string();
        Ok(Self {
            full_box_header,
            location,
            name,
        })
    }

    pub fn from_stream_header_url<T: Stream>(
        stream: &mut T,
        box_header: BoxHeader,
    ) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
        let mut location = String::new();
        if (full_box_header.flags() & 1) != 0 {
            location = stream.read_zero_term_string();
        }
        Ok(Self {
            full_box_header,
            location,
            name: String::new(),
        })
    }

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn location(&self) -> &String {
        &self.location
    }

    pub fn set_location(&mut self, loc: String) {
        self.location = loc;
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
