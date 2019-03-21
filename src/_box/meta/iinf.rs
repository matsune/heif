use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

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
    pub fn new<T: Stream>(stream: &mut T, full_box_header: FullBoxHeader) -> Result<ItemInfoBox> {
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
    pub fn new<T: Stream>(stream: &mut T, full_box_header: FullBoxHeader) -> Result<Self> {
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
            item_name = stream.read_zero_term_string();
            if item_type == "mime" {
                content_type = stream.read_zero_term_string();
                if stream.num_bytes_left() > 0 {
                    content_encoding = stream.read_zero_term_string();
                }
            } else if item_type == "uri " {
                item_uri_type = stream.read_zero_term_string();
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
        Self {
            content_location: String::new(),
            content_md5: String::new(),
            content_length: 0,
            transfer_length: 0,
            entry_count: 0,
            group_id: Vec::new(),
        }
    }

    fn new<T: Stream>(stream: &mut T) -> Result<Self> {
        let content_location = stream.read_zero_term_string();
        let content_md5 = stream.read_zero_term_string();
        let content_length = stream.read_8bytes()?.to_u64();
        let transfer_length = stream.read_8bytes()?.to_u64();
        let entry_count = stream.read_byte()?;
        let mut group_id = Vec::new();
        for _ in 0..entry_count {
            group_id.push(stream.read_4bytes()?.to_u32());
        }
        Ok(Self {
            content_location,
            content_md5,
            content_length,
            transfer_length,
            entry_count,
            group_id,
        })
    }
}
