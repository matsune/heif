pub mod hdlr;
pub mod iinf;
pub mod iloc;
pub mod iprp;
pub mod iref;
pub mod pitm;

use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

use hdlr::HandlerBox;
use iinf::ItemInfoBox;
use iloc::ItemLocationBox;
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

        while !stream.is_eof() {
            let child_box_header = BoxHeader::new(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            match child_box_header.box_type.as_str() {
                "hdlr" => {
                    let sub_fullbox_header = FullBoxHeader::new(&mut ex, child_box_header)?;
                    let b = HandlerBox::new(&mut ex, sub_fullbox_header)?;
                    println!(">>hdlr {:?}", b);
                    handler_box = Some(b);
                }
                "pitm" => {
                    let child_fullbox_header = FullBoxHeader::new(&mut ex, child_box_header)?;
                    let b = PrimaryItemBox::new(&mut ex, child_fullbox_header)?;
                    println!(">>pitm {:?}", b);
                    primary_item_box = Some(b);
                }
                "iloc" => {
                    let child_fullbox_header = FullBoxHeader::new(&mut ex, child_box_header)?;
                    let b = ItemLocationBox::new(&mut ex, child_fullbox_header)?;
                    println!(">>iloc {:?}", b);
                    item_location_box = Some(b);
                }
                "iinf" => {
                    let child_fullbox_header = FullBoxHeader::new(&mut ex, child_box_header)?;
                    let b = ItemInfoBox::new(&mut ex, child_fullbox_header)?;
                    println!(">>iinf {:?}", b);
                    item_info_box = Some(b);
                }
                "iref" => {
                    let child_fullbox_header = FullBoxHeader::new(&mut ex, child_box_header)?;
                    let b = ItemReferenceBox::new(&mut ex, child_fullbox_header)?;
                    println!(">>iref {:?}", b);
                    item_reference_box = Some(b);
                }
                "iprp" => {
                    let b = ItemPropertiesBox::new(&mut ex, child_box_header)?;
                    println!(">>iprp {:?}", b);
                    item_properties_box = Some(b);
                }
                _ => panic!("unimplemented {},", child_box_header.box_type),
            };
        }
        Ok(MetaBox {
            full_box_header,
            handler_box,
            primary_item_box,
            item_location_box,
            item_reference_box,
            item_properties_box,
        })
    }
}
