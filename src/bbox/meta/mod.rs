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

use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
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

#[derive(Debug, Default)]
pub struct MetaBox {
    pub full_box_header: FullBoxHeader,
    pub handler_box: Option<HandlerBox>,
    pub primary_item_box: Option<PrimaryItemBox>,
    pub item_location_box: Option<ItemLocationBox>,
    pub item_info_box: Option<ItemInfoBox>,
    pub item_reference_box: Option<ItemReferenceBox>,
    pub item_properties_box: Option<ItemPropertiesBox>,
    pub group_list_box: Option<GroupListBox>,
    pub data_information_box: Option<DataInformationBox>,
    pub item_data_box: Option<ItemDataBox>,
    pub item_protection_box: Option<ItemProtectionBox>,
}

impl MetaBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, header)?;
        let mut handler_box = Option::None;
        let mut primary_item_box = Option::None;
        let mut item_location_box = Option::None;
        let mut item_info_box = Option::None;
        let mut item_reference_box = Option::None;
        let mut item_properties_box = Option::None;
        let mut group_list_box = Option::None;
        let mut data_information_box = Option::None;
        let mut item_data_box = Option::None;
        let mut item_protection_box = Option::None;
        while !stream.is_eof() {
            let child_box_header = BoxHeader::from_stream(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            match child_box_header.box_type().to_string().as_str() {
                "hdlr" => {
                    handler_box = Some(HandlerBox::from_stream_header(&mut ex, child_box_header)?)
                }
                "pitm" => {
                    primary_item_box = Some(PrimaryItemBox::from_stream_header(
                        &mut ex,
                        child_box_header,
                    )?)
                }
                "iloc" => {
                    item_location_box = Some(ItemLocationBox::from_stream_header(
                        &mut ex,
                        child_box_header,
                    )?)
                }
                "iinf" => {
                    item_info_box =
                        Some(ItemInfoBox::from_stream_header(&mut ex, child_box_header)?)
                }
                "iref" => {
                    item_reference_box = Some(ItemReferenceBox::from_stream_header(
                        &mut ex,
                        child_box_header,
                    )?)
                }
                "iprp" => {
                    item_properties_box = Some(ItemPropertiesBox::from_stream_header(
                        &mut ex,
                        child_box_header,
                    )?)
                }
                "grpl" => {
                    group_list_box =
                        Some(GroupListBox::from_stream_header(&mut ex, child_box_header)?)
                }
                "dinf" => {
                    data_information_box = Some(DataInformationBox::from_stream_header(
                        &mut ex,
                        child_box_header,
                    )?)
                }
                "idat" => {
                    item_data_box =
                        Some(ItemDataBox::from_stream_header(&mut ex, child_box_header)?)
                }
                "ipro" => {
                    item_protection_box = Some(ItemProtectionBox::from_stream_header(
                        &mut ex,
                        child_box_header,
                    )?)
                }
                _ => {} //skip
            };
        }
        Ok(Self {
            full_box_header,
            handler_box,
            primary_item_box,
            item_location_box,
            item_info_box,
            item_reference_box,
            item_properties_box,
            group_list_box,
            data_information_box,
            item_data_box,
            item_protection_box,
        })
    }

    pub fn item_properties_map(&self) -> HashMap<u32, ItemFeature> {
        let map = HashMap::new();
        map
    }
}
