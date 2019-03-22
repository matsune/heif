use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
use crate::Result;

#[derive(Debug, Default)]
pub struct ItemInfoBox {
    pub full_box_header: FullBoxHeader,
    pub item_info_list: Vec<ItemInfoEntry>,
}

impl ItemInfoBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.full_box_header = FullBoxHeader::new(stream, box_header)?;
        s.parse(stream)
    }

    pub fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        let entry_count = if self.full_box_header.version == 0 {
            stream.read_2bytes()?.to_u32()
        } else {
            stream.read_4bytes()?.to_u32()
        };
        self.item_info_list.clear();
        for _ in 0..entry_count {
            let entry_box_header = BoxHeader::new(stream)?;
            self.item_info_list
                .push(ItemInfoEntry::new(stream, entry_box_header)?);
        }
        Ok(self)
    }

    pub fn clear(&mut self) {
        self.item_info_list.clear();
    }

    pub fn item_ids(&self) -> Vec<u32> {
        self.item_info_list.iter().map(|i| i.item_id).collect()
    }

    pub fn add_item_info_entry(&mut self, info_entry: ItemInfoEntry) {
        self.item_info_list.push(info_entry);
    }

    pub fn item_by_id(&self, id: u32) -> Option<&ItemInfoEntry> {
        self.item_info_list.iter().find(|i| i.item_id == id)
    }

    pub fn item_by_type(&self, item_type: Byte4) -> Option<&ItemInfoEntry> {
        self.item_info_list
            .iter()
            .find(|i| i.item_type == item_type)
    }
}

#[derive(Default, Debug)]
pub struct ItemInfoEntry {
    full_box_header: FullBoxHeader,
    item_id: u32,
    item_protection_index: u16,
    item_name: String,
    content_type: String,
    content_encoding: String,
    extension_type: String,
    item_info_extension: ItemInfoExtension,
    item_type: Byte4,
    item_uri_type: String,
}

impl ItemInfoEntry {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.full_box_header = FullBoxHeader::new(stream, box_header)?;
        s.parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        if self.full_box_header.version == 0 || self.full_box_header.version == 1 {
            self.item_id = stream.read_2bytes()?.to_u32();
            self.item_protection_index = stream.read_2bytes()?.to_u16();
        }

        if self.full_box_header.version == 1 {
            if stream.num_bytes_left() > 0 {
                self.extension_type = stream.read_4bytes()?.to_string();
            }
            if stream.num_bytes_left() > 0 {
                self.item_info_extension = ItemInfoExtension::new(stream)?;
            }
        }

        if self.full_box_header.version >= 2 {
            self.item_id = if self.full_box_header.version == 2 {
                stream.read_2bytes()?.to_u32()
            } else if self.full_box_header.version == 3 {
                stream.read_4bytes()?.to_u32()
            } else {
                0
            };
            self.item_protection_index = stream.read_2bytes()?.to_u16();
            self.item_type = stream.read_4bytes()?;
            self.item_name = stream.read_zero_term_string();
            if self.item_type == "mime" {
                self.content_type = stream.read_zero_term_string();
                if stream.num_bytes_left() > 0 {
                    self.content_encoding = stream.read_zero_term_string();
                }
            } else if self.item_type == "uri " {
                self.item_uri_type = stream.read_zero_term_string();
            }
        }
        Ok(self)
    }

    fn item_id(&self) -> &u32 {
        &self.item_id
    }

    fn set_item_id(&mut self, item_id: u32) {
        self.item_id = item_id;
    }

    fn item_protection_index(&self) -> &u16 {
        &self.item_protection_index
    }

    fn set_item_protection_index(&mut self, idx: u16) {
        self.item_protection_index = idx;
    }

    fn item_name(&self) -> &String {
        &self.item_name
    }

    fn set_item_name(&mut self, name: String) {
        self.item_name = name;
    }

    fn content_type(&self) -> &String {
        &self.content_type
    }

    fn set_content_type(&mut self, ctype: String) {
        self.content_type = ctype;
    }

    fn content_encoding(&self) -> &String {
        &self.content_encoding
    }

    fn set_content_encoding(&mut self, enc: String) {
        self.content_encoding = enc;
    }

    fn extension_type(&self) -> &String {
        &self.extension_type
    }

    fn set_extension_type(&mut self, exType: String) {
        self.extension_type = exType
    }

    fn item_type(&self) -> &Byte4 {
        &self.item_type
    }

    fn set_item_type(&mut self, iType: Byte4) {
        self.item_type = iType
    }

    fn item_uri_type(&self) -> &String {
        &self.item_uri_type
    }

    fn set_item_uri_type(&mut self, uType: String) {
        self.item_uri_type = uType;
    }
}

#[derive(Default, Debug)]
pub struct ItemInfoExtension {
    content_location: String,
    content_md5: String,
    content_length: u64,
    transfer_length: u64,
    entry_count: u8,
    group_id: Vec<u32>,
}

impl ItemInfoExtension {
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

    fn content_location(&self) -> &String {
        &self.content_location
    }

    fn set_content_location(&mut self, location: String) {
        self.content_location = location;
    }

    fn content_md5(&self) -> &String {
        &self.content_md5
    }

    fn set_content_md5(&mut self, md5: String) {
        self.content_md5 = md5;
    }

    fn content_length(&self) -> &u64 {
        &self.content_length
    }

    fn set_content_length(&mut self, length: u64) {
        self.content_length = length;
    }

    fn transfer_length(&self) -> &u64 {
        &self.transfer_length
    }

    fn set_transfer_length(&mut self, len: u64) {
        self.transfer_length = len;
    }

    fn entry_count(&self) -> &u8 {
        &self.entry_count
    }

    fn set_entry_count(&mut self, count: u8) {
        self.entry_count = count;
    }

    fn group_id_at(&self, index: usize) -> Option<&u32> {
        self.group_id.get(index)
    }

    fn set_group_id_at(&mut self, id: u32, index: usize) {
        self.group_id[index] = id;
    }
}
