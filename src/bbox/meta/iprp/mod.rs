mod hevc;
mod ispe;

use std::collections::HashMap;

use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
use crate::{HeifError, Result};
use hevc::HevcConfigurationBox;
use ispe::ImageSpatialExtentsProperty;

use std::str::FromStr;

#[derive(Debug)]
pub struct ItemPropertiesBox {
    box_header: BoxHeader,
    container: ItemPropertyContainer,
    association_boxes: Vec<ItemPropertyAssociation>,
}

impl Default for ItemPropertiesBox {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new(Byte4::from_str("iprp").unwrap()),
            container: ItemPropertyContainer::default(),
            association_boxes: Vec::new(),
        }
    }
}

impl ItemPropertiesBox {
    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let box_header = box_header;
        let container_box_header = BoxHeader::from(stream)?;
        let mut ex = stream.extract_from(&container_box_header)?;
        let container = ItemPropertyContainer::from(&mut ex, container_box_header)?;
        let mut association_boxes = Vec::new();
        while !stream.is_eof() {
            let sub_box_header = BoxHeader::from(stream)?;
            if sub_box_header.box_type() != "ipma" {
                return Err(HeifError::InvalidFormat);
            }
            let mut ex = stream.extract_from(&sub_box_header)?;
            association_boxes.push(ItemPropertyAssociation::from(&mut ex, sub_box_header)?);
        }
        Ok(Self {
            box_header,
            container,
            association_boxes,
        })
    }
}

pub trait ItemProperty {}

pub struct ItemPropertyContainer {
    box_header: BoxHeader,
    properties: Vec<Box<ItemProperty>>,
}

impl std::fmt::Debug for ItemPropertyContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ItemPropertyContainer {:?}", self.box_header)
    }
}

impl Default for ItemPropertyContainer {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new(Byte4::from_str("ipco").unwrap()),
            properties: Vec::new(),
        }
    }
}

impl ItemPropertyContainer {
    pub fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        if box_header.box_type() != "ipco" {
            // TODO: ?
        }
        let mut properties: Vec<Box<ItemProperty>> = Vec::new();
        while !stream.is_eof() {
            let sub_box_header = BoxHeader::from(stream)?;
            let mut ex = stream.extract_from(&sub_box_header)?;
            let property: Box<ItemProperty> = match sub_box_header.box_type().to_string().as_str() {
                "hvcC" => Box::new(HevcConfigurationBox::from(&mut ex, sub_box_header)?),
                "ispe" => Box::new(ImageSpatialExtentsProperty::from(&mut ex, sub_box_header)?),
                // TODO:
                _ => unimplemented!("itemprop {}", sub_box_header.box_type().to_string()),
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

impl Default for ItemPropertyAssociation {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new(Byte4::from_str("ipma").unwrap(), 0, 0),
            associations: HashMap::new(),
        }
    }
}

impl ItemPropertyAssociation {
    fn from<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from(stream, box_header)?;
        let mut associations = HashMap::new();
        let entry_count = stream.read_4bytes()?.to_u32();
        for _ in 0..entry_count {
            let item_id = if full_box_header.version() < 1 {
                stream.read_2bytes()?.to_u32()
            } else {
                stream.read_4bytes()?.to_u32()
            };

            let mut association_entries = AssociationEntries::new();
            let association_count = stream.read_byte()?;
            for _ in 0..association_count {
                let is_essential = stream.read_bits(1)? != 0;
                let index = if (full_box_header.flags() & 1) != 0 {
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
