use std::io::Result;

use crate::bbox::{BoxHeader, FullBoxHeader};
use crate::bit::BitStream;

#[derive(Debug)]
pub struct MetaBox {
    pub full_box_header: FullBoxHeader,
    pub handler_box: Option<HandlerBox>,
    pub primary_item_box: Option<PrimaryItemBox>,
    pub item_location_box: Option<ItemLocationBox>,
}

impl MetaBox {
    pub fn new(stream: &mut BitStream, header: BoxHeader) -> Result<MetaBox> {
        let full_box_header = FullBoxHeader::new(stream, header)?;
        let size = full_box_header.box_size() - u64::from(full_box_header.header_size());

        let mut handler_box = None;
        let mut primary_item_box = None;
        let mut item_location_box = None;
        let mut item_info_box = None;

        for _ in 0..size {
            let child_box_header = BoxHeader::new(stream)?;
            let child_fullbox_header = FullBoxHeader::new(stream, child_box_header)?;

            match child_fullbox_header.box_header.box_type.as_str() {
                "hdlr" => {
                    handler_box = Some(HandlerBox::new(stream, child_fullbox_header)?);
                    println!(">>hdlr {:?}", handler_box);
                }
                "pitm" => {
                    primary_item_box = Some(PrimaryItemBox::new(stream, child_fullbox_header)?);
                    println!(">>pitm {:?}", primary_item_box);
                }
                "iloc" => {
                    item_location_box = Some(ItemLocationBox::new(stream, child_fullbox_header)?);
                    println!(">>iloc {:?}", item_location_box);
                }
                "iinf" => {
                    item_info_box = Some(ItemInfoBox::new(stream, child_fullbox_header)?);
                    println!(">>iinf {:?}", item_info_box);
                }
                _ => panic!(
                    "unimplemented {},",
                    child_fullbox_header.box_header.box_type
                ),
            };
        }
        Ok(MetaBox {
            full_box_header,
            handler_box,
            primary_item_box,
            item_location_box,
        })
    }
}

#[derive(Debug)]
pub struct HandlerBox {
    pub full_box_header: FullBoxHeader,
    pub handler_type: String,
    pub name: String,
}

impl HandlerBox {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<HandlerBox> {
        stream.skip_bytes(4)?;
        let handler_type = stream.read_4bytes()?.to_string();
        stream.skip_bytes(12)?;
        let size = full_box_header.box_size() - u64::from(full_box_header.header_size());
        let mut name = Vec::new();
        for _ in 0..(size - 20) {
            let c = stream.read_byte()?;
            if c == 0 {
                break;
            }
            name.push(c);
        }
        let name = String::from_utf8(name).unwrap();
        Ok(HandlerBox {
            full_box_header,
            handler_type,
            name,
        })
    }
}

#[derive(Debug)]
pub struct PrimaryItemBox {
    pub full_box_header: FullBoxHeader,
    pub item_id: u32,
}

impl PrimaryItemBox {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<PrimaryItemBox> {
        let item_id = if full_box_header.version == 0 {
            stream.read_2bytes()?.to_u32()
        } else {
            stream.read_4bytes()?.to_u32()
        };
        Ok(PrimaryItemBox {
            full_box_header,
            item_id,
        })
    }
}

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

#[derive(Debug)]
pub struct ItemInfoBox {
    pub full_box_header: FullBoxHeader,
    pub item_info_list: Vec<ItemInfoEntry>,
}

#[derive(Debug)]
pub struct ItemInfoEntry {
    pub full_box_header: FullBoxHeader,
    pub item_id: u32,
    pub item_protection_index: u16,
    pub item_name: String,
    pub content_type: String,
    pub content_encoding: String,
    pub extension_type: String,
    pub item_info_extension: ItemInfoExtension,
    pub item_type: String,
    pub item_uri_type: String,
}

#[derive(Debug)]
pub struct ItemInfoExtension {
    pub content_location: String,
    pub content_md5: String,
    pub content_length: u64,
    pub transfer_length: u64,
    pub entry_count: u8,
    pub group_id: Vec<u32>,
}

impl ItemInfoBox {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<ItemInfoBox> {
        let entry_count = if full_box_header.version == 0 {
            stream.read_2bytes()?.to_u32()
        } else {
            stream.read_4bytes()?.to_u32()
        };
        let mut item_info_list = Vec::new();
        for _ in 0..entry_count {
            let entry_box_header = BoxHeader::new(stream)?;
            let entry_fullbox_header = FullBoxHeader::new(stream, entry_box_header)?;
            item_info_list.push(ItemInfoEntry::new(stream, entry_fullbox_header)?);
        }
        Ok(ItemInfoBox {
            full_box_header,
            item_info_list,
        })
    }
}

impl ItemInfoEntry {
    pub fn new(stream: &mut BitStream, full_box_header: FullBoxHeader) -> Result<Self> {
        let mut item_id = 0u32;
        let mut item_protection_index = 0u16;
        let mut item_name = String::new();
        let mut content_type = String::new();
        let mut content_encoding = String::new();
        let mut extension_type = String::new();
        let mut item_info_extension = ItemInfoExtension::default();
        let mut item_type = String::new();
        let mut item_uri_type = String::new();

        if full_box_header.version == 0 || full_box_header.version == 1 {
            item_id = stream.read_2bytes()?.to_u32();
            item_protection_index = stream.read_2bytes()?.to_u16();
        }
        if full_box_header.version == 1 {
            if stream.num_bytes_left() > 0 {
                extension_type = stream.read_4bytes()?.to_string();
            }
            if stream.num_bytes_left() > 0 {
                item_info_extension = ItemInfoExtension::new(stream)?;
            }
        }
        if full_box_header.version >= 2 {
            item_id = if full_box_header.version == 2 {
                 stream.read_2bytes()?.to_u32()
            } else if full_box_header.version == 3 {
                stream.read_4bytes()?.to_u32()
            } else {
                0
            };
            item_protection_index = stream.read_2bytes()?.to_u16();
            item_type = stream.read_4bytes()?.to_string();
            item_name = stream.read_zero_term_string()?;
            if item_type == "mime" {
                content_type = stream.read_zero_term_string()?;
                if stream.num_bytes_left() > 0 {
                    content_encoding = stream.read_zero_term_string()?;
                }
            } else if item_type == "uri " {
               item_uri_type = stream.read_zero_term_string()?;
            }
        }
        Ok(ItemInfoEntry {
            full_box_header,
            item_id,
            item_protection_index,
            item_name,
            content_type,
            content_encoding,
            extension_type,
            item_info_extension,
            item_type,
            item_uri_type,
        })
    }
}

impl ItemInfoExtension {
    fn default() -> Self {
        Self{
            content_location: String::new(),
            content_md5: String::new(),
            content_length: 0,
            transfer_length: 0,
            entry_count: 0,
            group_id: Vec::new(),
        }
    }

    fn new(stream: &mut BitStream) -> Result<Self> {
        let content_location = stream.read_zero_term_string()?;
        let content_md5 = stream.read_zero_term_string()?;
        let content_length = stream.read_8bytes()?.to_u64();
        let transfer_length = stream.read_8bytes()?.to_u64();
        let entry_count = stream.read_byte()?;
        let mut group_id = Vec::new();
        for _  in 0..entry_count     {
            group_id.push(stream.read_4bytes()?.to_u32());
        }
        Ok(Self{
            content_location,
            content_md5,
            content_length,
            transfer_length,
            entry_count,
            group_id,
        })
    }
}