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

use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
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
    pub fn new<T: Stream>(stream: &mut T, header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.full_box_header = FullBoxHeader::new(stream, header)?;
        s.parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        while !stream.is_eof() {
            let child_box_header = BoxHeader::new(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            match child_box_header.box_type.as_str() {
                "hdlr" => self.handler_box = Some(HandlerBox::new(&mut ex, child_box_header)?),
                "pitm" => {
                    self.primary_item_box = Some(PrimaryItemBox::new(&mut ex, child_box_header)?)
                }
                "iloc" => {
                    self.item_location_box = Some(ItemLocationBox::new(&mut ex, child_box_header)?)
                }
                "iinf" => self.item_info_box = Some(ItemInfoBox::new(&mut ex, child_box_header)?),
                "iref" => {
                    self.item_reference_box =
                        Some(ItemReferenceBox::new(&mut ex, child_box_header)?)
                }
                "iprp" => {
                    self.item_properties_box =
                        Some(ItemPropertiesBox::new(&mut ex, child_box_header)?)
                }
                "grpl" => self.group_list_box = Some(GroupListBox::new(&mut ex, child_box_header)?),
                "dinf" => {
                    self.data_information_box =
                        Some(DataInformationBox::new(&mut ex, child_box_header)?)
                }
                "idat" => self.item_data_box = Some(ItemDataBox::new(&mut ex, child_box_header)?),
                "ipro" => {
                    self.item_protection_box =
                        Some(ItemProtectionBox::new(&mut ex, child_box_header)?)
                }
                _ => {} //skip
            };
        }
        Ok(self)
    }
}
