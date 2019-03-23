use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
struct ItemLocationExtent {
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

#[derive(Debug)]
enum ConstructionMethod {
    FileOffset,
    IdatOffset,
    ItemOffset,
}

impl ConstructionMethod {
    fn new(n: usize) -> Self {
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

#[derive(Debug)]
pub struct ItemLocationBox {
    pub full_box_header: FullBoxHeader,
    pub offset_size: u8,
    pub length_size: u8,
    pub base_offset_size: u8,
    pub index_size: u8,
    pub locations: Vec<ItemLocation>,
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
}
