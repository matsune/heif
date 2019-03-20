pub mod hdlr;
//pub mod iinf;
//pub mod iloc;
//pub mod iprp;
//pub mod iref;
pub mod pitm;

use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

use hdlr::HandlerBox;
//use iinf::ItemInfoBox;
//use iloc::ItemLocationBox;
//use iprp::ItemPropertiesBox;
//use iref::ItemReferenceBox;
use pitm::PrimaryItemBox;

#[derive(Debug)]
pub struct MetaBox {
    pub full_box_header: FullBoxHeader,
    pub handler_box: Option<HandlerBox>,
    pub primary_item_box: Option<PrimaryItemBox>,
    //    pub item_location_box: Option<ItemLocationBox>,
    //    pub item_reference_box: Option<ItemReferenceBox>,
    //    pub item_properties_box: Option<ItemPropertiesBox>,
}

impl MetaBox {
    pub fn new<T: Stream>(stream: &mut T, header: BoxHeader) -> Result<MetaBox> {
        let full_box_header = FullBoxHeader::new(stream, header)?;

        let mut handler_box = None;
        let mut primary_item_box = None;
        //  let mut item_location_box = None;
        //  let mut item_info_box = None;
        //  let mut item_reference_box = None;
        //  let mut item_properties_box = None;

        while !stream.is_eof() {
            let child_box_header = BoxHeader::new(stream)?;
            let mut ex = stream.extract_from(&child_box_header)?;
            match child_box_header.box_type.as_str() {
                "hdlr" => {
                    let sub_fullbox_header = FullBoxHeader::new(&mut ex, child_box_header)?;
                    handler_box = Some(HandlerBox::new(&mut ex, sub_fullbox_header)?);
                    println!(">>hdlr {:?}", handler_box);
                }
                "pitm" => {
                    let child_fullbox_header = FullBoxHeader::new(&mut ex, child_box_header)?;
                    primary_item_box = Some(PrimaryItemBox::new(&mut ex, child_fullbox_header)?);
                    println!(">>pitm {:?}", primary_item_box);
                }
                //                "iloc" => {
                //                    let child_fullbox_header = FullBoxHeader::new(stream, child_box_header)?;
                //                    item_location_box = Some(ItemLocationBox::new(stream, child_fullbox_header)?);
                //                    println!(">>iloc {:?}", item_location_box);
                //                }
                //                "iinf" => {
                //                    let child_fullbox_header = FullBoxHeader::new(stream, child_box_header)?;
                //                    item_info_box = Some(ItemInfoBox::new(stream, child_fullbox_header)?);
                //                    println!(">>iinf {:?}", item_info_box);
                //                }
                //                "iref" => {
                //                    let child_fullbox_header = FullBoxHeader::new(stream, child_box_header)?;
                //                    item_reference_box = Some(ItemReferenceBox::new(stream, child_fullbox_header)?);
                //                    println!(">>iref {:?}", item_reference_box);
                //                }
                //                "iprp" => {
                //                    item_properties_box = Some(ItemPropertiesBox::new(stream, child_box_header)?);
                //                    println!(">>iprp {:?}", item_properties_box);
                //                }
                _ => panic!("unimplemented {},", child_box_header.box_type),
            };
        }
        Ok(MetaBox {
            full_box_header,
            handler_box,
            primary_item_box,
            //          item_location_box,
            //          item_reference_box,
            //          item_properties_box,
        })
    }
}
