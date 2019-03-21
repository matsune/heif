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

#[derive(Debug)]
pub struct MetaBox {
    pub full_box_header: FullBoxHeader,
    pub handler_box: Option<HandlerBox>,
    pub primary_item_box: Option<PrimaryItemBox>,
    pub item_location_box: Option<ItemLocationBox>,
    pub item_reference_box: Option<ItemReferenceBox>,
    pub item_properties_box: Option<ItemPropertiesBox>,
    pub group_list_box: Option<GroupListBox>,
    pub data_information_box: Option<DataInformationBox>,
    pub item_data_box: Option<ItemDataBox>,
    pub item_protection_box: Option<ItemProtectionBox>,
}

impl MetaBox {
    pub fn new<T: Stream>(stream: &mut T, header: BoxHeader) -> Result<MetaBox> {
        let full_box_header = FullBoxHeader::new(stream, header)?;

        let mut handler_box = None;
        let mut primary_item_box = None;
        let mut item_location_box = None;
        let mut item_info_box = None;
        let mut item_reference_box = None;
        let mut item_properties_box = None;
        let mut group_list_box = None;
        let mut data_information_box = None;
        let mut item_data_box = None;
        let mut item_protection_box = None;

        while !stream.is_eof() {
            let child_box_header = BoxHeader::new(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            match child_box_header.box_type.as_str() {
                "hdlr" => {
                    let b = HandlerBox::new(&mut ex, child_box_header)?;
                    println!(">>hdlr {:?}", b);
                    handler_box = Some(b);
                }
                "pitm" => {
                    let b = PrimaryItemBox::new(&mut ex, child_box_header)?;
                    println!(">>pitm {:?}", b);
                    primary_item_box = Some(b);
                }
                "iloc" => {
                    let b = ItemLocationBox::new(&mut ex, child_box_header)?;
                    println!(">>iloc {:?}", b);
                    item_location_box = Some(b);
                }
                "iinf" => {
                    let b = ItemInfoBox::new(&mut ex, child_box_header)?;
                    println!(">>iinf {:?}", b);
                    item_info_box = Some(b);
                }
                "iref" => {
                    let b = ItemReferenceBox::new(&mut ex, child_box_header)?;
                    println!(">>iref {:?}", b);
                    item_reference_box = Some(b);
                }
                "iprp" => {
                    let b = ItemPropertiesBox::new(&mut ex, child_box_header)?;
                    println!(">>iprp {:?}", b);
                    item_properties_box = Some(b);
                }
                "grpl" => {
                    let b = GroupListBox::new(&mut ex, child_box_header)?;
                    println!(">>grpl {:?}", b);
                    group_list_box = Some(b);
                }
                "dinf" => {
                    let b = DataInformationBox::new(&mut ex, child_box_header)?;
                    println!(">>dinf {:?}", b);
                    data_information_box = Some(b);
                }
                "idat" => {
                    let b = ItemDataBox::new(&mut ex, child_box_header)?;
                    println!(">>idat {:?}", b);
                    item_data_box = Some(b);
                }
                "ipro" => {
                    let b = ItemProtectionBox::new(&mut ex, child_box_header)?;
                    println!(">>ipro {:?}", b);
                    item_protection_box = Some(b);
                }
                _ => {} //skip
            };
        }
        Ok(MetaBox {
            full_box_header,
            handler_box,
            primary_item_box,
            item_location_box,
            item_reference_box,
            item_properties_box,
            group_list_box,
            data_information_box,
            item_data_box,
            item_protection_box,
        })
    }
}
