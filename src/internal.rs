use std::collections::{HashMap, HashSet};

use crate::bit::Byte4;
use crate::data::*;

#[derive(Debug, Default)]
pub struct FileFeature {
    file_feature_set: HashSet<FileFeatureEnum>,
}

impl FileFeature {
    pub fn has_feature(&self, feature: &FileFeatureEnum) -> bool {
        self.file_feature_set.contains(feature)
    }

    pub fn set_feature(&mut self, feature: FileFeatureEnum) {
        self.file_feature_set.insert(feature);
    }

    pub fn feature_mask(&self) -> u32 {
        let mut mask = 0u32;
        for set in &self.file_feature_set {
            mask |= *set as u32;
        }
        mask
    }
}

#[derive(Debug, Default)]
pub struct TrackFeature {
    track_feature_set: HashSet<TrackFeatureEnum>,
}

impl TrackFeature {
    pub fn has_feature(&self, feature: &TrackFeatureEnum) -> bool {
        self.track_feature_set.contains(feature)
    }

    pub fn set_feature(&mut self, feature: TrackFeatureEnum) {
        self.track_feature_set.insert(feature);
    }

    pub fn feature_mask(&self) -> u32 {
        let mut mask = 0u32;
        for set in &self.track_feature_set {
            mask |= *set as u32;
        }
        mask
    }
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

pub type IdVec = Vec<u32>;
pub type DataVec = Vec<u8>;
pub type TypeToIdsMap = HashMap<Byte4, IdVec>;
pub type Groupings = Vec<EntityGrouping>;
pub type ParameterSetMap = HashMap<DecoderSpecInfoType, DataVec>;
pub type PropertyTypeVector = Vec<ItemPropertyInfo>;

pub type ItemFeaturesMap = HashMap<u32, ItemFeature>;
pub type TrackPropertiesMap = HashMap<u32, TrackProperties>;
pub type SamplePropertiesMap = HashMap<u32, SampleProperties>;

#[derive(Debug, Default)]
pub struct MetaBoxProperties {
    pub context_id: u32,
    pub meta_box_feature: MetaBoxFeature,
    pub item_features_map: ItemFeaturesMap,
    pub entity_groupings: Groupings,
}

#[derive(Debug)]
pub struct MoovProperties {
    pub moov_id: u32,
    pub meta_box_properties: MetaBoxProperties,
}

#[derive(Debug)]
pub struct SampleProperties {
    pub sample_id: u32,
    pub sample_entry_type: Byte4,
    pub sample_description_index: u32,
    pub sample_type: SampleType,
    pub sample_duration_ts: u64,
    pub sample_composition_offset_ts: i64,
    pub has_clap: bool,
    pub has_auxi: bool,
    pub coding_constraints: CodingConstraints,
    pub size: u64,
}

#[derive(Debug)]
pub struct TrackProperties {
    pub track_id: u32,
    pub alternate_group_id: u32,
    pub track_feature: TrackFeature,
    pub sample_properties: HashMap<u32, SampleProperties>,
    pub alternate_track_ids: IdVec,
    pub reference_track_ids: HashMap<String, IdVec>,
    pub grouped_samples: Vec<SampleGrouping>,
    pub equivalences: Vec<SampleVisualEquivalence>,
    pub metadatas: Vec<SampleToMetadataItem>,
    pub reference_samples: Vec<DirectReferenceSamples>,
    pub max_sample_size: u64,
    pub time_scale: u32,
    pub edit_list: EditList,
}

#[derive(Debug, Default)]
pub struct FileInformationInternal {
    pub file_feature: FileFeature,
    pub track_properties: HashMap<u32, TrackProperties>,
    pub root_meta_box_properties: MetaBoxProperties,
    pub movie_timescale: u32,
}

// context_id, item_id
pub type Id = (u32, u32);
