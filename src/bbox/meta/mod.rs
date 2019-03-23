pub mod dinf;
pub mod grpl;
pub mod hdlr;
pub mod idat;
pub mod iinf;
pub mod iloc;
pub mod ipro;
pub mod iprp;
pub mod iref;
pub mod pitm;

use std::collections::HashMap;
use std::str::FromStr;

use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
use crate::reader::ItemFeature;
use crate::Result;

use dinf::DataInformationBox;
use grpl::GroupListBox;
use hdlr::HandlerBox;
use idat::ItemDataBox;
use iinf::ItemInfoBox;
use iloc::ItemLocationBox;
use ipro::ItemProtectionBox;
use iprp::ItemPropertiesBox;
use iref::ItemReferenceBox;
use pitm::PrimaryItemBox;

#[derive(Debug)]
pub struct MetaBox {
    full_box_header: FullBoxHeader,
    handler_box: HandlerBox,
    primary_item_box: PrimaryItemBox,
    item_location_box: ItemLocationBox,
    item_info_box: ItemInfoBox,
    item_reference_box: ItemReferenceBox,
    item_properties_box: ItemPropertiesBox,
    group_list_box: GroupListBox,
    data_information_box: DataInformationBox,
    item_data_box: ItemDataBox,
    item_protection_box: ItemProtectionBox,
}

impl Default for MetaBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("meta").unwrap(), 0, 0),
            handler_box: HandlerBox::default(),
            primary_item_box: PrimaryItemBox::default(),
            item_location_box: ItemLocationBox::default(),
            item_info_box: ItemInfoBox::default(),
            item_reference_box: ItemReferenceBox::default(),
            item_properties_box: ItemPropertiesBox::default(),
            group_list_box: GroupListBox::default(),
            data_information_box: DataInformationBox::default(),
            item_data_box: ItemDataBox::default(),
            item_protection_box: ItemProtectionBox::default(),
        }
    }
}

impl MetaBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.full_box_header = FullBoxHeader::from_stream_header(stream, header)?;
        s.parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        while !stream.is_eof() {
            let child_box_header = BoxHeader::from_stream(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            match child_box_header.box_type().to_string().as_str() {
                "hdlr" => {
                    self.handler_box = HandlerBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "pitm" => {
                    self.primary_item_box =
                        PrimaryItemBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "iloc" => {
                    self.item_location_box =
                        ItemLocationBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "iinf" => {
                    self.item_info_box =
                        ItemInfoBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "iref" => {
                    self.item_reference_box =
                        ItemReferenceBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "iprp" => {
                    self.item_properties_box =
                        ItemPropertiesBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "grpl" => {
                    self.group_list_box =
                        GroupListBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "dinf" => {
                    self.data_information_box =
                        DataInformationBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "idat" => {
                    self.item_data_box =
                        ItemDataBox::from_stream_header(&mut ex, child_box_header)?;
                }
                "ipro" => {
                    self.item_protection_box =
                        ItemProtectionBox::from_stream_header(&mut ex, child_box_header)?;
                }
                _ => {} //skip
            };
        }
        Ok(self)
    }

    pub fn item_properties_map(&self) -> HashMap<u32, ItemFeature> {
        let map = HashMap::new();
        map
    }
}
