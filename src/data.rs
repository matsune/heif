use crate::bit::Byte4;

#[derive(Debug)]
pub enum TrackSampleType {
    OutRef,
    OutNonRef,
    NonOutRef,
    Display,
    Samples,
}

#[derive(Debug)]
pub enum ItemPropertyType {
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
pub struct ItemPropertyInfo {
    pub item_property_type: ItemPropertyType,
    pub index: u32,
    pub is_essential: bool,
}

pub struct TimestampIDPair {
    pub timestamp: i64,
    pub item_id: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FileFeatureEnum {
    HasSingleImage = 1,
    HasImageCollection = 1 << 1,
    HasImageSequence = 1 << 2,
    HasRootLevelMetaBox = 1 << 3,
    HasAlternateTracks = 1 << 4,
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

pub struct DecoderConfiguration {
    pub decoder_config_id: u32,
    pub decoder_specific_info: Vec<DecoderSpecificInfo>,
}

type FeatureBitMask = u32;

#[derive(Debug)]
pub struct ItemInformation {
    pub item_id: u32,
    pub item_type: Byte4,
    pub description: ItemDescription,
    pub features: FeatureBitMask,
    pub size: u64,
}

#[derive(Debug, Default)]
pub struct EntityGrouping {
    pub group_type: Byte4,
    pub group_id: u32,
    pub entity_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct MetaBoxInformation {
    pub features: FeatureBitMask,
    pub item_informations: Vec<ItemInformation>,
    pub entity_groupings: Vec<EntityGrouping>,
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
pub struct Byte4ToIds {
    pub box_type: Byte4,
    pub track_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct SampleAndEntryIDs {
    pub sample_id: u32,
    pub sample_group_description_index: u32,
}

#[derive(Debug)]
pub struct SampleGrouping {
    pub grouping_type: Byte4,
    pub type_param: u32,
    pub samples: Vec<SampleAndEntryIDs>,
}

#[derive(Debug)]
pub struct SampleVisualEquivalence {
    pub sample_group_description_index: u32,
    pub time_offset: u16,
    pub timescale_multiplier: u16,
}

#[derive(Debug)]
pub struct SampleToMetadataItem {
    pub sample_group_description_index: u32,
    pub metadata_item_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct DirectReferenceSamples {
    pub sample_group_description_index: u32,
    pub sample_id: u32,
    pub reference_item_ids: Vec<u32>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SampleType {
    OutputNonReferenceFrame,
    OutputReferenceFrame,
    NnonOutputReferenceFrame,
}

#[derive(Debug)]
pub struct SampleInformation {
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
pub struct TrackInformation {
    pub track_id: u32,
    pub alternate_group_id: u32,
    pub features: FeatureBitMask,
    pub alternate_track_ids: Vec<u32>,
    pub reference_track_ids: Vec<Byte4ToIds>,
    pub sample_groups: Vec<SampleGrouping>,
    pub sample_properties: Vec<SampleInformation>,
    pub equivalences: Vec<SampleVisualEquivalence>,
    pub metadatas: Vec<SampleToMetadataItem>,
    pub reference_samples: Vec<DirectReferenceSamples>,
    pub max_sample_size: u64,
    pub time_scale: u32,
    pub edit_list: EditList,
}

#[derive(Debug)]
pub struct FileInformation {
    pub features: FeatureBitMask,
    pub root_meta_box_information: MetaBoxInformation,
    pub track_information: Vec<TrackInformation>,
    pub movie_timescale: u32,
}

// common

#[derive(Debug)]
pub struct Rational {
    pub num: u64,
    pub den: u64,
}

#[derive(Debug)]
pub enum DecoderSpecInfoType {
    AvcSPS = 7,
    AvcPPS = 8,
    HevcVPS = 32,
    HevcSPS = 33,
    HevcPPS = 34,

    AudioSpecificConfig,
}

#[derive(Debug)]
pub struct DecoderSpecificInfo {
    pub dec_spec_info_type: DecoderSpecInfoType,
    pub dec_spec_info_data: Vec<u8>,
}

#[derive(Debug)]
pub struct ItemDescription {
    pub name: String,
    pub content_type: String,
    pub content_encoding: String,
}

#[derive(Debug)]
pub struct Mirror {
    pub horizontal_axis: bool,
}

#[derive(Debug)]
pub struct Rotate {
    pub angle: u32,
}

#[derive(Debug)]
pub struct RelativeLocation {
    pub horizontal_offset: u32,
    pub vertical_offset: u32,
}

#[derive(Debug)]
pub struct PixelAspectRatio {
    pub relative_width: u32,
    pub relative_height: u32,
}

#[derive(Debug)]
pub struct PixelInformation {
    pub bits_per_channel: Vec<u8>,
}

#[derive(Debug)]
pub struct ColorInformation {
    pub color_type: Byte4,
    pub color_primaries: u16,
    pub transfer_characteristics: u16,
    pub matrix_coefficients: u16,
    pub full_range_flag: bool,
    pub icc_profile: Vec<u8>,
}

pub struct CleanAperture {
    pub width_n: u32,
    pub width_d: u32,
    pub height_n: u32,
    pub height_d: u32,
    pub horizontal_offset_n: u32,
    pub horizontal_offset_d: u32,
    pub vertical_offset_n: u32,
    pub vertical_offset_d: u32,
}

pub struct AuxiliaryType {
    pub aux_type: String,
    pub sub_type: String,
}

pub struct RawProperty {
    pub raw_type: Byte4,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Offset {
    pub horizontal: i32,
    pub vertical: i32,
}

#[derive(Debug)]
pub struct Overlay {
    pub rgba: (u16, u16, u16, u16),
    pub output_width: u32,
    pub output_height: u32,
    pub offsets: Vec<Offset>,
    pub image_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct Grid {
    pub output_width: u32,
    pub output_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub image_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct CodingConstraints {
    pub all_ref_pics_intra: bool,
    pub intra_pred_used: bool,
    pub max_ref_per_pic: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditType {
    Empty,
    Dwell,
    Shift,
}

#[derive(Debug)]
pub struct EditUnit {
    pub edit_type: EditType,
    pub media_time_in_track_ts: i64,
    pub duration_in_movie_ts: u64,
}

#[derive(Debug)]
pub struct EditList {
    pub looping: bool,
    pub repetitions: f64,
    pub edit_units: Vec<EditUnit>,
}
