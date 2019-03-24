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

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::{Byte4, Stream};
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

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn set_full_box_header(&mut self, full_box_header: FullBoxHeader) {
        self.full_box_header = full_box_header;
    }

    pub fn handler_box(&self) -> &HandlerBox {
        &self.handler_box
    }

    pub fn set_handler_box(&mut self, handler_box: HandlerBox) {
        self.handler_box = handler_box;
    }

    pub fn primary_item_box(&self) -> &PrimaryItemBox {
        &self.primary_item_box
    }

    pub fn set_primary_item_box(&mut self, primary_item_box: PrimaryItemBox) {
        self.primary_item_box = primary_item_box;
    }

    pub fn item_location_box(&self) -> &ItemLocationBox {
        &self.item_location_box
    }

    pub fn set_item_location_box(&mut self, item_location_box: ItemLocationBox) {
        self.item_location_box = item_location_box;
    }

    pub fn item_info_box(&self) -> &ItemInfoBox {
        &self.item_info_box
    }

    pub fn set_item_info_box(&mut self, item_info_box: ItemInfoBox) {
        self.item_info_box = item_info_box;
    }

    pub fn item_reference_box(&self) -> &ItemReferenceBox {
        &self.item_reference_box
    }

    pub fn set_item_reference_box(&mut self, item_reference_box: ItemReferenceBox) {
        self.item_reference_box = item_reference_box;
    }

    pub fn item_properties_box(&self) -> &ItemPropertiesBox {
        &self.item_properties_box
    }

    pub fn set_item_properties_box(&mut self, item_properties_box: ItemPropertiesBox) {
        self.item_properties_box = item_properties_box;
    }

    pub fn group_list_box(&self) -> &GroupListBox {
        &self.group_list_box
    }

    pub fn set_group_list_box(&mut self, group_list_box: GroupListBox) {
        self.group_list_box = group_list_box;
    }

    pub fn data_information_box(&self) -> &DataInformationBox {
        &self.data_information_box
    }

    pub fn set_data_information_box(&mut self, data_information_box: DataInformationBox) {
        self.data_information_box = data_information_box;
    }

    pub fn item_data_box(&self) -> &ItemDataBox {
        &self.item_data_box
    }

    pub fn set_item_data_box(&mut self, item_data_box: ItemDataBox) {
        self.item_data_box = item_data_box;
    }

    pub fn item_protection_box(&self) -> &ItemProtectionBox {
        &self.item_protection_box
    }

    pub fn set_item_protection_box(&mut self, item_protection_box: ItemProtectionBox) {
        self.item_protection_box = item_protection_box;
    }
}

pub type ItemFeaturesMap = HashMap<u32, ItemFeature>;

#[derive(Debug, Default)]
pub struct MetaBoxProperties {
    context_id: u32,
    meta_box_feature: MetaBoxFeature,
    item_features_map: ItemFeaturesMap,
    entity_groupings: Vec<EntityGrouping>,
}

#[derive(Debug, Default)]
pub struct MetaBoxFeature {
    meta_box_feature_set: HashSet<MetaBoxFeatureEnum>,
}

impl MetaBoxFeature {
    pub fn has_feature(&self, feature: &MetaBoxFeatureEnum) -> bool {
        self.meta_box_feature_set.contains(feature)
    }

    pub fn set_feature(&mut self, feature: MetaBoxFeatureEnum) {
        self.meta_box_feature_set.insert(feature);
    }

    pub fn feature_mask(&self) -> u32 {
        let mut mask = 0u32;
        for set in &self.meta_box_feature_set {
            mask |= *set as u32;
        }
        mask
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MetaBoxFeatureEnum {
    IsSingleImage = 1,
    IsImageCollection = 1 << 1,
    HasMasterImages = 1 << 2,
    HasThumbnails = 1 << 3,
    HasAuxiliaryImages = 1 << 4,
    HasDerivedImages = 1 << 5,
    HasPreComputedDerivedImages = 1 << 6,
    HasHiddenImages = 1 << 7,
    HasGroupLists = 1 << 8,
}

#[derive(Debug, Default)]
pub struct ItemFeature {
    item_feature_set: HashSet<ItemFeatureEnum>,
}

impl ItemFeature {
    pub fn has_feature(&self, feature: &ItemFeatureEnum) -> bool {
        self.item_feature_set.contains(feature)
    }

    pub fn set_feature(&mut self, feature: ItemFeatureEnum) {
        self.item_feature_set.insert(feature);
    }

    pub fn feature_mask(&self) -> u32 {
        let mut mask = 0u32;
        for set in &self.item_feature_set {
            mask |= *set as u32;
        }
        mask
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ItemFeatureEnum {
    IsMasterImage = 1,
    IsThumbnailImage = 1 << 1,
    IsAuxiliaryImage = 1 << 2,
    IsPrimaryImage = 1 << 3,
    IsDerivedImage = 1 << 4,
    IsPreComputedDerivedImage = 1 << 5,
    IsHiddenImage = 1 << 6,
    IsCoverImage = 1 << 7,
    IsProtected = 1 << 8,
    HasLinkedThumbnails = 1 << 9,
    HasLinkedAuxiliaryImage = 1 << 10,
    HasLinkedDerivedImage = 1 << 11,
    HasLinkedPreComputedDerivedImage = 1 << 12,
    HasLinkedTiles = 1 << 13,
    HasLinkedMetadata = 1 << 14,
    IsTileImageItem = 1 << 15,
    IsMetadataItem = 1 << 16,
    IsExifItem = 1 << 17,
    IsXMPItem = 1 << 18,
    IsMPEG7Item = 1 << 19,
}

#[derive(Debug, Default)]
pub struct EntityGrouping {
    group_type: Byte4,
    group_id: u32,
    entity_ids: Vec<u32>,
}

pub fn extract_metabox_properties(metabox: &MetaBox) -> MetaBoxProperties {
    MetaBoxProperties {
        context_id: 0,
        meta_box_feature: extract_metabox_feature(metabox),
        item_features_map: extract_metabox_item_properties_map(metabox),
        entity_groupings: Vec::new(),
    }
}

pub fn extract_metabox_item_properties_map(metabox: &MetaBox) -> HashMap<u32, ItemFeature> {
    let mut map = HashMap::new();
    let item_ids = metabox.item_info_box().item_ids();
    for item_id in item_ids {
        let item = metabox.item_info_box().item_by_id(item_id);
        if item.is_none() {
            continue;
        }
        let item = item.unwrap();
        let mut item_features = ItemFeature::default();
        let item_type = item.item_type();
        if is_image_item_type(item_type) {

        } else {
            if item.item_protection_index() > 0 {
                item_features.set_feature(ItemFeatureEnum::IsProtected);
            }
            // TODO
        }
    }
    map
}

pub fn extract_metabox_feature(metabox: &MetaBox) -> MetaBoxFeature {
    MetaBoxFeature::default()
}

pub fn is_image_item_type(item_type: &Byte4) -> bool {
    let item_type = item_type.to_string();
    item_type == "avc1"
        || item_type == "hvc1"
        || item_type == "grid"
        || item_type == "iovl"
        || item_type == "iden"
        || item_type == "jpeg"
}
