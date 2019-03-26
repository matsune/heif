use crate::bbox::header::{BoxHeader, FullBoxHeader, Header};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct ItemLocationExtent {
    extent_index: usize,
    extent_offset: usize,
    extent_length: usize,
}

impl Default for ItemLocationExtent {
    fn default() -> Self {
        Self {
            extent_index: 0,
            extent_offset: 0,
            extent_length: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConstructionMethod {
    FileOffset,
    IdatOffset,
    ItemOffset,
}

impl ConstructionMethod {
    pub fn new(n: usize) -> Self {
        match n {
            1 => ConstructionMethod::IdatOffset,
            2 => ConstructionMethod::ItemOffset,
            _ => ConstructionMethod::FileOffset,
        }
    }
}

#[derive(Debug)]
pub struct ItemLocation {
    item_id: u32,
    method: ConstructionMethod,
    data_ref_index: u16,
    base_offset: usize,
    extent_list: Vec<ItemLocationExtent>,
}

impl Default for ItemLocation {
    fn default() -> Self {
        Self {
            item_id: 0,
            method: ConstructionMethod::FileOffset,
            data_ref_index: 0,
            base_offset: 0,
            extent_list: Vec::new(),
        }
    }
}

impl ItemLocation {
    pub fn item_id(&self) -> u32 {
        self.item_id
    }

    pub fn set_item_id(&mut self, id: u32) {
        self.item_id = id;
    }

    pub fn method(&self) -> &ConstructionMethod {
        &self.method
    }

    pub fn set_method(&mut self, m: ConstructionMethod) {
        self.method = m;
    }

    pub fn data_ref_index(&self) -> u16 {
        self.data_ref_index
    }

    pub fn set_data_ref_index(&mut self, idx: u16) {
        self.data_ref_index = idx;
    }

    pub fn base_offset(&self) -> usize {
        self.base_offset
    }

    pub fn set_base_offset(&mut self, n: usize) {
        self.base_offset = n;
    }

    pub fn extent_list(&self) -> &Vec<ItemLocationExtent> {
        &self.extent_list
    }

    pub fn add_extent(&mut self, ex: ItemLocationExtent) {
        self.extent_list.push(ex);
    }
}

#[derive(Debug)]
pub struct ItemLocationBox {
    full_box_header: FullBoxHeader,
    offset_size: u8,
    length_size: u8,
    base_offset_size: u8,
    index_size: u8,
    locations: Vec<ItemLocation>,
}

impl Default for ItemLocationBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("iloc").unwrap(), 0, 0),
            offset_size: 4,
            length_size: 4,
            base_offset_size: 4,
            index_size: 0,
            locations: Vec::new(),
        }
    }
}

impl BBox for ItemLocationBox {
    type HeaderType = FullBoxHeader;
    fn header(&self) -> &Self::HeaderType {
        &self.full_box_header
    }
}

impl ItemLocationBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
        let offset_size = stream.read_bits(4)? as u8;
        let length_size = stream.read_bits(4)? as u8;
        let base_offset_size = stream.read_bits(4)? as u8;
        let index_size = if full_box_header.version() == 1 || full_box_header.version() == 2 {
            stream.read_bits(4)? as u8
        } else {
            stream.read_bits(4)?;
            0
        };

        let item_count = if full_box_header.version() < 2 {
            stream.read_2bytes()?.to_u32()
        } else if full_box_header.version() == 2 {
            stream.read_4bytes()?.to_u32()
        } else {
            0
        };

        let mut locations = Vec::new();
        for _ in 0..item_count {
            let mut item_loc = ItemLocation::default();
            item_loc.item_id = if full_box_header.version() < 2 {
                stream.read_2bytes()?.to_u32()
            } else if full_box_header.version() == 2 {
                stream.read_4bytes()?.to_u32()
            } else {
                0u32
            };

            if full_box_header.version() == 1 || full_box_header.version() == 2 {
                stream.read_bits(12)?;
                item_loc.method = ConstructionMethod::new(stream.read_bits(4)?);
            }
            item_loc.data_ref_index = stream.read_2bytes()?.to_u16();
            item_loc.base_offset = stream.read_bits(usize::from(base_offset_size) * 8)?;
            let extent_count = stream.read_2bytes()?.to_u16();
            for _ in 0..extent_count {
                let mut loc_ext = ItemLocationExtent::default();
                if (full_box_header.version() == 1 || full_box_header.version() == 2)
                    && index_size > 0
                {
                    loc_ext.extent_index = stream.read_bits(usize::from(index_size) * 8)?;
                }
                loc_ext.extent_offset = stream.read_bits(usize::from(offset_size * 8))?;
                loc_ext.extent_length = stream.read_bits(usize::from(length_size * 8))?;
                item_loc.extent_list.push(loc_ext);
            }
            locations.push(item_loc);
        }
        Ok(Self {
            full_box_header,
            offset_size,
            length_size,
            base_offset_size,
            index_size,
            locations,
        })
    }

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn offset_size(&self) -> u8 {
        self.offset_size
    }

    pub fn set_offset_size(&mut self, n: u8) {
        self.offset_size = n;
    }

    pub fn length_size(&self) -> u8 {
        self.length_size
    }

    pub fn set_length_size(&mut self, n: u8) {
        self.length_size = n;
    }

    pub fn base_offset_size(&self) -> u8 {
        self.base_offset_size
    }

    pub fn set_base_offset_size(&mut self, n: u8) {
        self.base_offset_size = n;
    }

    pub fn locations_count(&self) -> usize {
        self.locations.len()
    }

    pub fn add_location(&mut self, loc: ItemLocation) {
        if *loc.method() != ConstructionMethod::FileOffset {
            self.full_box_header.set_version(1);
        }
        self.locations.push(loc);
    }
}
