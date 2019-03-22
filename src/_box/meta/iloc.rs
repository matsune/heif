use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
struct ItemLocationExtent {
    extent_index: usize,
    extent_offset: usize,
    extent_length: usize,
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
    base_offset: usize,
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

#[derive(Debug, Default)]
pub struct ItemLocationBox {
    pub full_box_header: FullBoxHeader,
    pub offset_size: u8,
    pub length_size: u8,
    pub base_offset_size: u8,
    pub index_size: u8,
    pub locations: Vec<ItemLocation>,
}

impl ItemLocationBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.full_box_header = FullBoxHeader::new(stream, box_header)?;
        s.parse(stream)
    }

    pub fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        self.offset_size = stream.read_bits(4)? as u8;
        self.length_size = stream.read_bits(4)? as u8;
        self.base_offset_size = stream.read_bits(4)? as u8;
        self.index_size = if self.full_box_header.version == 1 || self.full_box_header.version == 2
        {
            stream.read_bits(4)? as u8
        } else {
            stream.read_bits(4)?;
            0
        };

        let item_count = if self.full_box_header.version < 2 {
            stream.read_2bytes()?.to_u32()
        } else if self.full_box_header.version == 2 {
            stream.read_4bytes()?.to_u32()
        } else {
            0
        };

        self.locations.clear();
        for _ in 0..item_count {
            let mut item_loc = ItemLocation::new();
            item_loc.item_id = if self.full_box_header.version < 2 {
                stream.read_2bytes()?.to_u32()
            } else if self.full_box_header.version == 2 {
                stream.read_4bytes()?.to_u32()
            } else {
                0u32
            };

            if self.full_box_header.version == 1 || self.full_box_header.version == 2 {
                stream.read_bits(12)?;
                item_loc.method = match stream.read_bits(4)? {
                    1 => ConstructionMethod::IdatOffset,
                    2 => ConstructionMethod::ItemOffset,
                    _ => ConstructionMethod::FileOffset,
                };
            }
            item_loc.data_ref_index = stream.read_2bytes()?.to_u16();
            item_loc.base_offset = stream.read_bits(usize::from(self.base_offset_size) * 8)?;
            let extent_count = stream.read_2bytes()?.to_u16();
            for _ in 0..extent_count {
                let mut loc_ext = ItemLocationExtent::new();
                if (self.full_box_header.version == 1 || self.full_box_header.version == 2)
                    && self.index_size > 0
                {
                    loc_ext.extent_index = stream.read_bits(usize::from(self.index_size) * 8)?;
                }
                loc_ext.extent_offset = stream.read_bits(usize::from(self.offset_size * 8))?;
                loc_ext.extent_length = stream.read_bits(usize::from(self.length_size * 8))?;
                item_loc.extent_list.push(loc_ext);
            }
            self.locations.push(item_loc);
        }
        Ok(self)
    }
}
