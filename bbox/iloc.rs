use crate::bbox::FullBoxHeader;
use crate::bit::BitStream;
use std::io::Result;

#[derive(Debug)]
struct ItemLocationExtent {
    extent_index: u64,
    extent_offset: u64,
    extent_length: u64,
}

impl ItemLocationExtent {
    fn new() -> ItemLocationExtent {
        ItemLocationExtent {
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

#[derive(Debug)]
pub struct ItemLocation {
    item_id: u32,
    method: ConstructionMethod,
    data_ref_index: u16,
    base_offset: u64,
    extent_list: Vec<ItemLocationExtent>,
}

impl ItemLocation {
    fn new() -> Self {
        ItemLocation {
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

impl ItemLocationBox {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<ItemLocationBox> {
        let offset_size = stream.read_bits(4)? as u8;
        let length_size = stream.read_bits(4)? as u8;
        let base_offset_size = stream.read_bits(4)? as u8;
        let index_size = if full_box_header.version == 1 || full_box_header.version == 2 {
            stream.read_bits(4)? as u8
        } else {
            stream.read_bits(4)?;
            0
        };

        let item_count = if full_box_header.version < 2 {
            stream.read_2bytes()?.to_u32()
        } else if full_box_header.version == 2 {
            stream.read_4bytes()?.to_u32()
        } else {
            0
        };

        let mut locations = Vec::new();
        for _ in 0..item_count {
            let mut item_loc = ItemLocation::new();
            item_loc.item_id = if full_box_header.version < 2 {
                stream.read_2bytes()?.to_u32()
            } else if full_box_header.version == 2 {
                stream.read_4bytes()?.to_u32()
            } else {
                0u32
            };

            if full_box_header.version == 1 || full_box_header.version == 2 {
                stream.read_bits(12)?;
                item_loc.method = match stream.read_bits(4)? {
                    1 => ConstructionMethod::IdatOffset,
                    2 => ConstructionMethod::ItemOffset,
                    _ => ConstructionMethod::FileOffset,
                };
            }
            item_loc.data_ref_index = stream.read_2bytes()?.to_u16();
            item_loc.base_offset = u64::from(stream.read_bits(u32::from(base_offset_size * 8))?);
            let extent_count = stream.read_2bytes()?.to_u16();
            for _ in 0..extent_count {
                let mut loc_ext = ItemLocationExtent::new();
                if (full_box_header.version == 1 || full_box_header.version == 2) && index_size > 0
                {
                    loc_ext.extent_index = u64::from(stream.read_bits(u32::from(index_size * 8))?);
                }
                loc_ext.extent_offset = u64::from(stream.read_bits(u32::from(offset_size * 8))?);
                loc_ext.extent_length = u64::from(stream.read_bits(u32::from(length_size * 8))?);
                item_loc.extent_list.push(loc_ext);
            }
            locations.push(item_loc);
        }
        Ok(ItemLocationBox {
            full_box_header,
            offset_size,
            length_size,
            base_offset_size,
            index_size,
            locations,
        })
    }
}
