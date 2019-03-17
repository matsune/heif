pub mod hdlr;
pub mod iinf;
pub mod iloc;
pub mod iref;
pub mod pitm;

use crate::bbox::{BoxHeader, FullBoxHeader};
use crate::bit::BitStream;
use std::io::Result;

use hdlr::HandlerBox;
use iinf::ItemInfoBox;
use iloc::ItemLocationBox;
use iref::ItemReferenceBox;
use pitm::PrimaryItemBox;

#[derive(Debug)]
pub struct MetaBox {
    pub full_box_header: FullBoxHeader,
    pub handler_box: Option<HandlerBox>,
    pub primary_item_box: Option<PrimaryItemBox>,
    pub item_location_box: Option<ItemLocationBox>,
    pub item_reference_box: Option<ItemReferenceBox>,
}

impl MetaBox {
    pub fn new(stream: &mut BitStream, header: BoxHeader) -> Result<MetaBox> {
        let full_box_header = FullBoxHeader::new(stream, header)?;
        let size = full_box_header.box_size() - u64::from(full_box_header.header_size());

        let mut handler_box = None;
        let mut primary_item_box = None;
        let mut item_location_box = None;
        let mut item_info_box = None;
        let mut item_reference_box = None;

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
                "iref" => {
                    item_reference_box = Some(ItemReferenceBox::new(stream, child_fullbox_header)?);
                    println!(">>iref {:?}", item_reference_box);
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
            item_reference_box,
        })
    }
}
