use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::error::HeifError;
use crate::Result;

pub struct DataInformationBox {
    pub box_header: BoxHeader,
    pub data_reference_box: Option<DataReferenceBox>,
}

impl std::fmt::Debug for DataInformationBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DataInformationBox {:?}", self.box_header)
    }
}

impl DataInformationBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let data_reference_box = if stream.is_eof() {
            None
        } else {
            let mut ex = stream.extract_from(&box_header)?;
            let child_box_header = BoxHeader::new(&mut ex)?;
            Some(DataReferenceBox::new(&mut ex, child_box_header)?)
        };
        Ok(Self {
            box_header,
            data_reference_box,
        })
    }
}

pub struct DataReferenceBox {
    full_box_header: FullBoxHeader,
    data_entries: Vec<Box<DataEntry>>,
}

impl DataReferenceBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let entry_count = stream.read_4bytes()?.to_u32();
        let mut data_entries = Vec::new();
        for _ in 0..entry_count {
            let child_box_header = BoxHeader::new(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            let data_entry: Box<DataEntry> = match child_box_header.box_type.as_str() {
                "urn " => Box::new(DataEntryUrnBox::new(&mut ex, child_box_header)?),
                "url " => Box::new(DataEntryUrlBox::new(&mut ex, child_box_header)?),
                _ => return Err(HeifError::InvalidFormat),
            };
            data_entries.push(data_entry);
        }
        Ok(Self {
            full_box_header,
            data_entries,
        })
    }
}

pub trait DataEntry {}

#[derive(Debug)]
pub struct DataEntryBox {
    full_box_header: FullBoxHeader,
    location: String,
}

impl DataEntry for DataEntryBox {}

pub struct DataEntryUrnBox {
    full_box_header: FullBoxHeader,
    location: String,
    name: String,
}
impl DataEntryUrnBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let name = stream.read_zero_term_string();
        let location = stream.read_zero_term_string();
        Ok(Self {
            full_box_header,
            name,
            location,
        })
    }
}

impl DataEntry for DataEntryUrnBox {}

pub struct DataEntryUrlBox {
    full_box_header: FullBoxHeader,
    location: String,
}

impl DataEntryUrlBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let mut location = String::new();
        if (full_box_header.flags & 1) != 0 {
            location = stream.read_zero_term_string();
        }
        Ok(Self {
            full_box_header,
            location,
        })
    }
}

impl DataEntry for DataEntryUrlBox {}
