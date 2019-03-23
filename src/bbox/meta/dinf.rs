use crate::bbox::header::{BoxHeader, FullBoxHeader};
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

impl DataInformationBox {
    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let data_reference_box = if stream.is_eof() {
            DataReferenceBox::default()
        } else {
            let mut ex = stream.extract_from(&box_header)?;
            let child_box_header = BoxHeader::from(&mut ex)?;
            DataReferenceBox::from(&mut ex, child_box_header)?
        };
        Ok(Self {
            box_header,
            data_reference_box,
        })
    }

    // TODO: add_entry
    // pub fn add_entry<T: DataEntry>(&mut self, entry: T) {
    //     if self.data_reference_box.is_none() {
    //         self.data_reference_box = Some(DataReferenceBox::default())
    //     }
    //     self.data_reference_box.add
    // }
}

pub struct DataReferenceBox {
    full_box_header: FullBoxHeader,
    data_entries: Vec<Box<DataEntry>>,
}

impl std::fmt::Debug for DataReferenceBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DataReferenceBox {:?}", self.full_box_header)
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
    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        Self {
            full_box_header: FullBoxHeader::from(stream, box_header)?,
            data_entries: Vec::new(),
        }
        .parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        let entry_count = stream.read_4bytes()?.to_u32();
        self.data_entries.clear();
        for _ in 0..entry_count {
            let child_box_header = BoxHeader::from(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            let data_entry: Box<DataEntry> = match child_box_header.box_type().to_string().as_str()
            {
                "urn " => Box::new(DataEntryUrnBox::from(&mut ex, child_box_header)?),
                "url " => Box::new(DataEntryUrlBox::from(&mut ex, child_box_header)?),
                _ => return Err(HeifError::InvalidFormat),
            };
            self.data_entries.push(data_entry);
        }
        Ok(self)
    }

    // pub fn add_entry<T: DataEntry>(&mut self, entry: T) {
    //     self.data_entries.push(Box::new(entry));
    // }
}

pub trait DataEntry {}

#[derive(Debug)]
pub struct DataEntryBox {
    full_box_header: FullBoxHeader,
    location: String,
}

impl DataEntryBox {
    pub fn new(box_type: Byte4, version: u8, flags: u32) -> Self {
        Self {
            full_box_header: FullBoxHeader::new(box_type, version, flags),
            location: String::new(),
        }
    }

    pub fn location(&self) -> &String {
        &self.location
    }

    pub fn set_location(&mut self, loc: String) {
        self.location = loc;
    }
}

impl DataEntry for DataEntryBox {}

pub struct DataEntryUrnBox {
    full_box_header: FullBoxHeader,
    location: String,
    name: String,
}

impl Default for DataEntryUrnBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("urn ").unwrap(), 0, 0),
            location: String::new(),
            name: String::new(),
        }
    }
}

impl DataEntryUrnBox {
    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from(stream, box_header)?;
        let name = stream.read_zero_term_string();
        let location = stream.read_zero_term_string();
        Ok(Self {
            full_box_header,
            name,
            location,
        })
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

impl DataEntry for DataEntryUrnBox {}

pub struct DataEntryUrlBox {
    full_box_header: FullBoxHeader,
    location: String,
}

impl DataEntryUrlBox {
    pub fn new(is_self_contained: bool) -> Self {
        let flags = if is_self_contained { 1 } else { 0 };
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("url ").unwrap(), 0, flags),
            location: String::new(),
        }
    }

    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from(stream, box_header)?;
        let mut location = String::new();
        if (full_box_header.flags() & 1) != 0 {
            location = stream.read_zero_term_string();
        }
        Ok(Self {
            full_box_header,
            location,
        })
    }

    pub fn location(&self) -> &String {
        &self.location
    }

    pub fn set_location(&mut self, loc: String) {
        self.location = loc;
    }
}

impl DataEntry for DataEntryUrlBox {}
