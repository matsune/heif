mod hevc;
mod ispe;

use std::collections::HashMap;

use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::error::HeifError;
use crate::Result;
use hevc::HevcConfigurationBox;
use ispe::ImageSpatialExtentsProperty;

#[derive(Debug, Default)]
pub struct ItemPropertiesBox {
    pub box_header: BoxHeader,
    pub container: ItemPropertyContainer,
    pub association_boxes: Vec<ItemPropertyAssociation>,
}

impl ItemPropertiesBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut s = Self::default();
        s.box_header = box_header;
        s.parse(stream)
    }

    fn parse<T: Stream>(mut self, stream: &mut T) -> Result<Self> {
        let container_box_header = BoxHeader::new(stream)?;
        let mut ex = stream.extract_from(&container_box_header)?;
        self.container = ItemPropertyContainer::new(&mut ex, container_box_header)?;
        self.association_boxes.clear();
        while !stream.is_eof() {
            let sub_box_header = BoxHeader::new(stream)?;
            if sub_box_header.box_type != "ipma" {
                return Err(HeifError::InvalidFormat);
            }
            let mut ex = stream.extract_from(&sub_box_header)?;
            self.association_boxes
                .push(ItemPropertyAssociation::new(&mut ex, sub_box_header)?);
        }
        Ok(self)
    }
}

pub trait ItemProperty {}

#[derive(Default)]
pub struct ItemPropertyContainer {
    pub box_header: BoxHeader,
    pub properties: Vec<Box<ItemProperty>>,
}

impl std::fmt::Debug for ItemPropertyContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ItemPropertyContainer {:?}", self.box_header)
    }
}

impl ItemPropertyContainer {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        if box_header.box_type != "ipco" {
            // TODO: ?
        }
        let mut properties: Vec<Box<ItemProperty>> = Vec::new();
        while !stream.is_eof() {
            let sub_box_header = BoxHeader::new(stream)?;
            let mut ex = stream.extract_from(&sub_box_header)?;
            let property: Box<ItemProperty> = match sub_box_header.box_type.as_str() {
                "hvcC" => Box::new(HevcConfigurationBox::new(&mut ex, sub_box_header)?),
                "ispe" => Box::new(ImageSpatialExtentsProperty::new(&mut ex, sub_box_header)?),
                // TODO:
                _ => unimplemented!("itemprop {}", sub_box_header.box_type),
            };
            properties.push(property);
        }
        Ok(Self {
            box_header,
            properties,
        })
    }
}

type AssociationID = u32;
type AssociationEntries = Vec<AssociationEntry>;

#[derive(Debug)]
pub struct AssociationEntry {
    is_essential: bool,
    index: u16,
}

#[derive(Debug)]
pub struct ItemPropertyAssociation {
    full_box_header: FullBoxHeader,
    associations: HashMap<AssociationID, AssociationEntries>,
}

const PROPERTY_INDEX_WIDTH_LARGE: usize = 15;
const PROPERTY_INDEX_WIDTH_SMALL: usize = 7;

impl ItemPropertyAssociation {
    fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let mut associations = HashMap::new();
        let entry_count = stream.read_4bytes()?.to_u32();
        for _ in 0..entry_count {
            let item_id = if full_box_header.version < 1 {
                stream.read_2bytes()?.to_u32()
            } else {
                stream.read_4bytes()?.to_u32()
            };

            let mut association_entries = AssociationEntries::new();
            let association_count = stream.read_byte()?;
            for _ in 0..association_count {
                let is_essential = stream.read_bits(1)? != 0;
                let index = if (full_box_header.flags & 1) != 0 {
                    stream.read_bits(PROPERTY_INDEX_WIDTH_LARGE)? as u16
                } else {
                    stream.read_bits(PROPERTY_INDEX_WIDTH_SMALL)? as u16
                };
                association_entries.push(AssociationEntry {
                    is_essential,
                    index,
                });
            }
            associations.insert(item_id, association_entries);
        }
        Ok(Self {
            full_box_header,
            associations,
        })
    }
}
