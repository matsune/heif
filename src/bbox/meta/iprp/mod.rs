pub mod hevc;
pub mod ispe;

use std::collections::HashMap;

use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::{HeifError, Result};
use hevc::HevcConfigurationBox;
use ispe::ImageSpatialExtentsProperty;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DecoderParameterType {
    AvcSPS,
    AvcPPS,
    HevcVPS,
    HevcSPS,
    HevcPPS,
    AudioSpecificConfig,
}

pub type ConfigurationMap = HashMap<DecoderParameterType, Vec<u8>>;

pub trait DecoderConfigurationRecord {
    fn configuration_map(&self) -> ConfigurationMap;
}

#[derive(Debug)]
pub struct ItemPropertiesBox {
    box_header: BoxHeader,
    container: ItemPropertyContainer,
    association_boxes: Vec<ItemPropertyAssociation>,
}

impl Default for ItemPropertiesBox {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new("iprp".parse().unwrap()),
            container: ItemPropertyContainer::default(),
            association_boxes: Vec::new(),
        }
    }
}

impl BBox for ItemPropertiesBox {
    fn box_type(&self) -> &Byte4 {
        self.box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemPropertiesBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let box_header = box_header;
        let container_box_header = BoxHeader::from_stream(stream)?;
        let mut ex = stream.extract_from(&container_box_header)?;
        let container = ItemPropertyContainer::from_stream_header(&mut ex, container_box_header)?;
        let mut association_boxes = Vec::new();
        while !stream.is_eof() {
            let sub_box_header = BoxHeader::from_stream(stream)?;
            if sub_box_header.box_type() != "ipma" {
                return Err(HeifError::Unknown(
                    "ItemPropertiesBox includes a box which is not ipma",
                ));
            }
            let mut ex = stream.extract_from(&sub_box_header)?;
            association_boxes.push(ItemPropertyAssociation::from_stream_header(
                &mut ex,
                sub_box_header,
            )?);
        }
        Ok(Self {
            box_header,
            container,
            association_boxes,
        })
    }

    pub fn box_header(&self) -> &BoxHeader {
        &self.box_header
    }

    pub fn property_by_index(&self, idx: usize) -> Option<&Box<BBox>> {
        self.container.property_at(idx)
    }

    pub fn find_property_index(&self, p_type: PropertyType, item_id: u32) -> u32 {
        for ipma in &self.association_boxes {
            if let Some(association_entries) = ipma.get_association_entries(item_id) {
                for entry in association_entries {
                    if let Some(item_property) =
                        self.container.property_at(entry.index as usize - 1)
                    {
                        if self.get_property_type(item_property) == p_type {
                            return u32::from(entry.index);
                        }
                    }
                }
            }
        }
        0
    }

    fn get_property_type(&self, property: &Box<BBox>) -> PropertyType {
        match property.box_type().to_string().as_str() {
            "auxC" => PropertyType::AUXC,
            "avcC" => PropertyType::AVCC,
            "clap" => PropertyType::CLAP,
            "colr" => PropertyType::COLR,
            "free" => PropertyType::FREE,
            "hvcC" => PropertyType::HVCC,
            "imir" => PropertyType::IMIR,
            "irot" => PropertyType::IROT,
            "ispe" => PropertyType::ISPE,
            "jpgC" => PropertyType::JPGC,
            "pasp" => PropertyType::PASP,
            "pixi" => PropertyType::PIXI,
            "rloc" => PropertyType::RLOC,
            "skip" => PropertyType::FREE,
            _ => PropertyType::RAW,
        }
    }

    pub fn get_item_properties(&self, item_id: u32) -> Result<PropertyInfos> {
        let mut property_info_vec = PropertyInfos::new();
        for ipma in &self.association_boxes {
            if let Some(associations) = ipma.get_association_entries(item_id) {
                for entry in associations {
                    if entry.index == 0 {
                        continue;
                    }
                    let index = entry.index as usize - 1;
                    let item_property = match self.container.property_at(index) {
                        Some(i) => i,
                        None => return Err(HeifError::Unknown("invalid property index")),
                    };
                    let property_type = self.get_property_type(item_property);
                    if property_type == PropertyType::FREE {
                        continue;
                    }
                    property_info_vec.push(PropertyInfo {
                        property_type,
                        index,
                        is_essential: entry.is_essential,
                    });
                }
                if !associations.is_empty() {
                    break;
                }
            }
        }
        Ok(property_info_vec)
    }
}

#[derive(Debug)]
pub struct PropertyInfo {
    pub property_type: PropertyType,
    pub index: usize,
    pub is_essential: bool,
}

pub type PropertyInfos = Vec<PropertyInfo>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PropertyType {
    RAW,
    AUXC,
    AVCC,
    CLAP,
    COLR,
    FREE,
    HVCC,
    IMIR,
    IROT,
    ISPE,
    JPGC,
    PASP,
    PIXI,
    RLOC,
}

pub struct ItemPropertyContainer {
    box_header: BoxHeader,
    properties: Vec<Box<BBox>>,
}

impl std::fmt::Debug for ItemPropertyContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ItemPropertyContainer {:?}", self.box_header)
    }
}

impl Default for ItemPropertyContainer {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new("ipco".parse().unwrap()),
            properties: Vec::new(),
        }
    }
}

impl BBox for ItemPropertyContainer {
    fn box_type(&self) -> &Byte4 {
        self.box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemPropertyContainer {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        if box_header.box_type() != "ipco" {
            // TODO: ?
        }
        let mut properties: Vec<Box<BBox>> = Vec::new();
        while !stream.is_eof() {
            let sub_box_header = BoxHeader::from_stream(stream)?;
            let mut ex = stream.extract_from(&sub_box_header)?;
            let property: Box<BBox> = match sub_box_header.box_type().to_string().as_str() {
                "hvcC" => Box::new(HevcConfigurationBox::from_stream_header(
                    &mut ex,
                    sub_box_header,
                )?),
                "ispe" => Box::new(ImageSpatialExtentsProperty::from_stream_header(
                    &mut ex,
                    sub_box_header,
                )?),
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

    pub fn property_at(&self, index: usize) -> Option<&Box<BBox>> {
        self.properties.get(index)
    }

    pub fn add_property(&mut self, prop: Box<BBox>) {
        self.properties.push(prop);
    }
}

type AssociationEntries = Vec<AssociationEntry>;

#[derive(Debug)]
pub struct AssociationEntry {
    is_essential: bool,
    index: u16,
}

#[derive(Debug)]
pub struct ItemPropertyAssociation {
    full_box_header: FullBoxHeader,
    associations: HashMap<u32, AssociationEntries>,
}

const PROPERTY_INDEX_WIDTH_LARGE: usize = 15;
const PROPERTY_INDEX_WIDTH_SMALL: usize = 7;

impl Default for ItemPropertyAssociation {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new("ipma".parse().unwrap(), 0, 0),
            associations: HashMap::new(),
        }
    }
}

impl BBox for ItemPropertyAssociation {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemPropertyAssociation {
    fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
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

    pub fn add_entry(&mut self, item_id: u32, index: u16, is_essential: bool) {
        let entries = vec![AssociationEntry {
            is_essential,
            index,
        }];
        self.associations.insert(item_id, entries);
        if self.full_box_header.version() == 0 && item_id > std::u16::MAX.into() {
            self.full_box_header.set_version(1);
        }
        if (self.full_box_header.flags() & 1) == 0 && index > 127 {
            self.full_box_header
                .set_flags(self.full_box_header.flags() | 1);
        }
    }

    pub fn get_association_entries(&self, item_id: u32) -> Option<&AssociationEntries> {
        self.associations.get(&item_id)
    }
}
