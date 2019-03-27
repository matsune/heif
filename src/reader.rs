use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::str::FromStr;

use crate::bbox::ftyp::FileTypeBox;
use crate::bbox::header::{BoxHeader, Header};
use crate::bbox::meta::iprp::PropertyType;
use crate::bbox::meta::MetaBox;
use crate::bbox::moov::MovieBox;
use crate::bit::{BitStream, Byte4, Stream};
use crate::{HeifError, Result};

type MetaBoxMap = HashMap<u32, MetaBox>;

#[derive(Debug, Default)]
pub struct HeifReader {
    ftyp: FileTypeBox,
    metabox_map: MetaBoxMap,
    metabox_info: HashMap<u32, MetaBoxInfo>,
    file_properties: FileInformationInternal,
}

impl HeifReader {
    pub fn load(&mut self, file_path: &str) -> Result<&mut Self> {
        let mut file = File::open(file_path).map_err(|_| HeifError::FileOpen)?;
        let mut stream = BitStream::from(&mut file)?;
        let mut ftyp_found = false;
        let mut metabox_found = false;
        let mut movie_found = false;

        self.metabox_map.clear();
        self.metabox_info.clear();
        self.file_properties = FileInformationInternal::default();

        while !stream.is_eof() {
            let header = BoxHeader::from_stream(&mut stream)?;
            let box_type = header.box_type();
            if box_type == "ftyp" {
                if ftyp_found {
                    return Err(HeifError::InvalidFormat);
                }
                ftyp_found = true;
                let mut ex = stream.extract_from(&header)?;
                self.ftyp = FileTypeBox::new(&mut ex, header)?;
            } else if box_type == "meta" {
                if metabox_found {
                    return Err(HeifError::InvalidFormat);
                }
                metabox_found = true;
                let mut ex = stream.extract_from(&header)?;
                self.metabox_map
                    .insert(0, MetaBox::from_stream_header(&mut ex, header)?);
                let metabox = self.metabox_map.get(&0).unwrap();
                self.file_properties.root_meta_box_properties =
                    self.extract_metabox_properties(&metabox);
                self.metabox_info.insert(0, self.extract_items(&metabox, 0));

                self.process_decoder_config_properties(&0);
            } else if box_type == "moov" {
                if movie_found {
                    return Err(HeifError::InvalidFormat);
                }
                movie_found = true;
                let mut ex = stream.extract_from(&header)?;
                let movie_box = MovieBox::new(&mut ex, header)?;
            } else if box_type == "mdat" || box_type == "free" || box_type == "skip" {
                println!(">>SKIPPING {}", box_type.to_string());
                stream.skip_bytes(header.body_size() as usize)?;
            } else {
                println!("unknown type {}", box_type.to_string());
                stream.skip_bytes(header.body_size() as usize)?;
            }
        }
        if !ftyp_found || (!metabox_found && !movie_found) {
            return Err(HeifError::InvalidFormat);
        }
        Ok(self)
    }

    fn extract_metabox_properties(&self, metabox: &MetaBox) -> MetaBoxProperties {
        let item_features_map = self.extract_metabox_item_properties_map(metabox);
        let entity_groupings = self.extract_metabox_entity_to_group_maps(metabox);
        let meta_box_feature = self.extract_metabox_feature(&item_features_map, &entity_groupings);
        MetaBoxProperties {
            context_id: 0,
            meta_box_feature,
            item_features_map,
            entity_groupings,
        }
    }

    fn extract_metabox_entity_to_group_maps(&self, metabox: &MetaBox) -> Vec<EntityGrouping> {
        let mut groupings = Vec::new();
        for group_box in metabox.group_list_box().entity_to_group_box_vector() {
            groupings.push(EntityGrouping {
                group_id: group_box.group_id(),
                group_type: group_box.full_box_header().box_type().clone(),
                entity_ids: group_box.entity_ids().to_vec(),
            })
        }
        groupings
    }

    fn extract_metabox_item_properties_map(&self, metabox: &MetaBox) -> HashMap<u32, ItemFeature> {
        let mut map = HashMap::new();
        let item_ids = metabox.item_info_box().item_ids();
        for item_id in item_ids {
            let item = metabox.item_info_box().item_by_id(&item_id);
            if item.is_none() {
                continue;
            }
            let item = item.unwrap();
            let mut item_features = ItemFeature::default();
            let item_type = item.item_type();
            if self.is_image_item_type(item_type) {
                if item.item_protection_index() > 0 {
                    item_features.set_feature(ItemFeatureEnum::IsProtected);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("thmb").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::IsThumbnailImage);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("auxl").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::IsAuxiliaryImage);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("base").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::IsPreComputedDerivedImage);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("dimg").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::IsDerivedImage);
                }
                if !item_features.has_feature(&ItemFeatureEnum::IsThumbnailImage)
                    && !item_features.has_feature(&ItemFeatureEnum::IsAuxiliaryImage)
                {
                    item_features.set_feature(ItemFeatureEnum::IsMasterImage);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("thmb").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::HasLinkedThumbnails);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("auxl").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::HasLinkedAuxiliaryImage);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("cdsc").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::HasLinkedMetadata);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("base").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::HasLinkedPreComputedDerivedImage);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("tbas").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::HasLinkedTiles);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("dimg").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::HasLinkedDerivedImage);
                }

                if metabox.primary_item_box().item_id() == item_id {
                    item_features.set_feature(ItemFeatureEnum::IsPrimaryImage);
                    item_features.set_feature(ItemFeatureEnum::IsCoverImage);
                }

                if (item.full_box_header().flags() & 0x1) != 0 {
                    item_features.set_feature(ItemFeatureEnum::IsHiddenImage);
                }
            } else {
                if item.item_protection_index() > 0 {
                    item_features.set_feature(ItemFeatureEnum::IsProtected);
                }
                if self.do_references_from_item_id_exist(
                    metabox,
                    item_id,
                    Byte4::from_str("cdsc").unwrap(),
                ) {
                    item_features.set_feature(ItemFeatureEnum::IsMetadataItem);
                }
                if item_type == "Exif" {
                    item_features.set_feature(ItemFeatureEnum::IsExifItem);
                } else if item_type == "mime" {
                    if item.content_type() == "application/rdf+xml" {
                        item_features.set_feature(ItemFeatureEnum::IsXMPItem);
                    } else {
                        item_features.set_feature(ItemFeatureEnum::IsMPEG7Item);
                    }
                } else if item_type == "hvt1" {
                    item_features.set_feature(ItemFeatureEnum::IsTileImageItem);
                }
            }
            map.insert(item_id, item_features);
        }
        map
    }

    fn extract_metabox_feature(
        &self,
        image_features: &ItemFeaturesMap,
        groupings: &Vec<EntityGrouping>,
    ) -> MetaBoxFeature {
        let mut meta_box_feature = MetaBoxFeature::default();
        if groupings.len() > 0 {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::HasGroupLists);
        }
        if image_features.len() == 1 {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::IsSingleImage);
        } else if image_features.len() > 1 {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::IsImageCollection);
        }

        for i in image_features {
            let features = i.1;
            if features.has_feature(&ItemFeatureEnum::IsMasterImage) {
                meta_box_feature.set_feature(MetaBoxFeatureEnum::HasMasterImages);
            }
            if features.has_feature(&ItemFeatureEnum::IsThumbnailImage) {
                meta_box_feature.set_feature(MetaBoxFeatureEnum::HasThumbnails);
            }
            if features.has_feature(&ItemFeatureEnum::IsAuxiliaryImage) {
                meta_box_feature.set_feature(MetaBoxFeatureEnum::HasAuxiliaryImages);
            }
            if features.has_feature(&ItemFeatureEnum::IsDerivedImage) {
                meta_box_feature.set_feature(MetaBoxFeatureEnum::HasDerivedImages);
            }
            if features.has_feature(&ItemFeatureEnum::IsPreComputedDerivedImage) {
                meta_box_feature.set_feature(MetaBoxFeatureEnum::HasPreComputedDerivedImages);
            }
            if features.has_feature(&ItemFeatureEnum::IsHiddenImage) {
                meta_box_feature.set_feature(MetaBoxFeatureEnum::HasHiddenImages);
            }
        }
        meta_box_feature
    }

    fn is_image_item_type(&self, item_type: &Byte4) -> bool {
        let item_type = item_type.to_string();
        item_type == "avc1"
            || item_type == "hvc1"
            || item_type == "grid"
            || item_type == "iovl"
            || item_type == "iden"
            || item_type == "jpeg"
    }

    fn do_references_from_item_id_exist(
        &self,
        metabox: &MetaBox,
        item_id: u32,
        ref_type: Byte4,
    ) -> bool {
        let refs = metabox.item_reference_box().references_of_type(ref_type);
        refs.iter().find(|r| r.from_item_id() == item_id).is_some()
    }

    fn extract_items(&self, metabox: &MetaBox, context_id: u32) -> MetaBoxInfo {
        let mut metabox_info = MetaBoxInfo::default();
        for item in metabox.item_info_box().item_info_list() {
            if item.item_type() == "grid" || item.item_type() == "iovl" {
                let i_protected = match self.get_protection(item.item_id()) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                if i_protected {
                    continue;
                }
                // TODO
                unimplemented!("extract_items");
            }
        }
        metabox_info
    }

    fn get_protection(&self, item_id: &u32) -> Result<bool> {
        let entry = match self
            .metabox_map
            .get(&self.file_properties.root_meta_box_properties.context_id)
        {
            Some(e) => e,
            None => return Err(HeifError::InvalidItemID),
        };
        let entry = match entry.item_info_box().item_by_id(item_id) {
            Some(e) => e,
            None => return Err(HeifError::InvalidItemID),
        };
        Ok(entry.item_protection_index() > 0)
    }

    pub fn process_decoder_config_properties(&self, context_id: &u32) {
        if let Some(iprp) = self.metabox_map.get(context_id) {
            let iprp = iprp.item_properties_box();
            for image_properties in &self
                .file_properties
                .root_meta_box_properties
                .item_features_map
            {
                let image_id = image_properties.0;
                let hvcc_index = iprp.find_property_index(PropertyType::HVCC, &image_id);
                let avcc_index = iprp.find_property_index(PropertyType::AVCC, &image_id);
                let mut config_index = (0, 0);
                let mut ty = Byte4::default();
                if hvcc_index != 0 {
                    config_index = (*context_id, hvcc_index);
                    ty = Byte4::from_str("hvc1").unwrap();
                } else if avcc_index != 0 {
                    config_index = (*context_id, avcc_index);
                    ty = Byte4::from_str("avc1").unwrap();
                } else {
                    continue;
                }

                // TODO
                // DecoderConfigurationBox child may be HevcConfigurationBox or AvcConfigurationBox
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct FileInformationInternal {
    file_feature: FileFeature,
    track_properties: HashMap<u32, TrackProperties>,
    root_meta_box_properties: MetaBoxProperties,
    movie_timescale: u32,
}

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FileFeatureEnum {
    HasSingleImage = 1,
    HasImageCollection = 1 << 1,
    HasImageSequence = 1 << 2,
    HasRootLevelMetaBox = 1 << 3,
    HasAlternateTracks = 1 << 4,
}

type IdVec = Vec<u32>;

#[derive(Debug)]
pub struct TrackProperties {
    track_id: u32,
    alternate_group_id: u32,
    track_feature: TrackFeature,
    sample_properties: HashMap<u32, SampleProperties>,
    alternate_track_ids: IdVec,
    reference_track_ids: HashMap<String, IdVec>,
    grouped_samples: Vec<SampleGrouping>,
    equivalences: Vec<SampleVisualEquivalence>,
    metadatas: Vec<SampleToMetadataItem>,
    reference_samples: Vec<DirectReferenceSamples>,
    max_sample_size: u64,
    time_scale: u32,
    edit_list: EditList,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TrackFeatureEnum {
    IsMasterImageSequence = 1,
    IsThumbnailImageSequence = 1 << 1,
    IsAuxiliaryImageSequence = 1 << 2,
    IsEnabled = 1 << 3,
    IsInMovie = 1 << 4,
    IsInPreview = 1 << 5,
    HasAlternatives = 1 << 6,
    HasCodingConstraints = 1 << 7,
    HasSampleGroups = 1 << 8,
    HasLinkedAuxiliaryImageSequence = 1 << 9,
    HasLinkedThumbnailImageSequence = 1 << 10,
    HasSampleToItemGrouping = 1 << 11,
    HasExifSampleEntry = 1 << 12,
    HasXmlSampleEntry = 1 << 13,
    HasEditList = 1 << 14,
    HasInfiniteLoopPlayback = 1 << 15,
    HasSampleEquivalenceGrouping = 1 << 16,
    IsAudioTrack = 1 << 17,
    IsVideoTrack = 1 << 18,
    DisplayAllSamples = 1 << 19,
}

#[derive(Debug)]
pub struct SampleProperties {
    sample_id: u32,
    sample_entry_type: Byte4,
    sample_description_index: u32,
    sample_type: SampleType,
    sample_duration_ts: u64,
    sample_composition_offset_ts: i64,
    has_clap: bool,
    has_auxi: bool,
    coding_constraints: CodingConstraints,
    size: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SampleType {
    OutputNonReferenceFrame,
    OutputReferenceFrame,
    NnonOutputReferenceFrame,
}

#[derive(Debug)]
pub struct CodingConstraints {
    all_ref_pics_intra: bool,
    intra_pred_used: bool,
    max_ref_per_pic: u8,
}

#[derive(Debug)]
pub struct SampleAndEntryIDs {
    sample_id: u32,
    sample_group_description_index: u32,
}

#[derive(Debug)]
pub struct SampleGrouping {
    grouping_type: Byte4,
    type_param: u32,
    samples: Vec<SampleAndEntryIDs>,
}

#[derive(Debug)]
pub struct SampleVisualEquivalence {
    sample_group_description_index: u32,
    time_offset: u16,
    timescale_multiplier: u16,
}

#[derive(Debug)]
pub struct SampleToMetadataItem {
    sample_group_description_index: u32,
    metadata_item_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct DirectReferenceSamples {
    sample_group_description_index: u32,
    sample_id: u32,
    reference_item_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct EditList {
    looping: bool,
    repetitions: f64,
    edit_units: Vec<EditUnit>,
}

#[derive(Debug)]
pub struct EditUnit {
    edit_type: EditType,
    media_time_in_track_ts: i64,
    duration_in_movie_ts: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditType {
    Empty,
    Dwell,
    Shift,
}

pub type ItemFeaturesMap = HashMap<u32, ItemFeature>;

#[derive(Debug, Default)]
struct MetaBoxProperties {
    pub context_id: u32,
    pub meta_box_feature: MetaBoxFeature,
    pub item_features_map: ItemFeaturesMap,
    pub entity_groupings: Vec<EntityGrouping>,
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
struct ItemFeature {
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
enum ItemFeatureEnum {
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

#[derive(Debug)]
pub struct ItemInfo {
    item_type: Byte4,
    name: String,
    content_type: String,
    content_encoding: String,
    width: u32,
    height: u32,
    display_time: u64,
}

type ItemInfoMap = HashMap<u32, ItemInfo>;

#[derive(Debug)]
enum ItemPropertyType {
    INVALID,

    RAW,
    AUXC,
    AVCC,
    CLAP,
    COLR,
    HVCC,
    IMIR,
    IROT,
    ISPE,
    JPGC,
    PASP,
    PIXI,
    RLOC,
}

#[derive(Debug)]
struct ItemPropertyInfo {
    item_property_type: ItemPropertyType,
    index: u32,
    is_essential: bool,
}

type PropertyTypeVector = Vec<ItemPropertyInfo>;

#[derive(Default, Debug)]
pub struct MetaBoxInfo {
    displayable_master_images: usize,
    item_info_map: ItemInfoMap,
    grid_items: HashMap<u32, Grid>,
    iovl_items: HashMap<u32, Overlay>,
    properties: HashMap<u32, PropertyTypeVector>,
}

#[derive(Debug)]
pub struct Grid {
    output_width: u32,
    output_height: u32,
    columns: u32,
    rows: u32,
    image_ids: Vec<u32>,
}

#[derive(Debug)]
struct Offset {
    horizontal: i32,
    vertical: i32,
}

#[derive(Debug)]
pub struct Overlay {
    rgba: (u16, u16, u16, u16),
    output_width: u32,
    output_height: u32,
    offsets: Vec<Offset>,
    image_ids: Vec<u32>,
}
