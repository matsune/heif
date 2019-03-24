use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::bbox::ftyp::FileTypeBox;
use crate::bbox::header::{BoxHeader, Header};
use crate::bbox::meta;
use crate::bbox::moov::MovieBox;
use crate::bit::{BitStream, Byte4, Stream};
use crate::{HeifError, Result};

#[derive(Debug, Default)]
pub struct HeifReader {
    ftyp: FileTypeBox,
    metabox_map: HashMap<u32, meta::MetaBox>,
    // file_properties: FileInformationInternal,
}

impl HeifReader {
    pub fn new(file_path: &str) -> Result<Self> {
        let mut file = File::open(file_path).map_err(|_| HeifError::FileOpen)?;
        let mut stream = BitStream::from(&mut file)?;
        let mut ftyp = Option::<FileTypeBox>::None;
        let mut metabox_map = HashMap::new();
        let mut movie_box = Option::<MovieBox>::None;
        let mut file_properties = FileInformationInternal::default();

        while !stream.is_eof() {
            let header = BoxHeader::from_stream(&mut stream)?;
            let box_type = header.box_type();
            if box_type == "ftyp" {
                if ftyp.is_some() {
                    return Err(HeifError::InvalidFormat);
                }
                let mut ex = stream.extract_from(&header)?;
                ftyp = Some(FileTypeBox::new(&mut ex, header)?);
            } else if box_type == "meta" {
                if !metabox_map.is_empty() {
                    return Err(HeifError::InvalidFormat);
                }
                let mut ex = stream.extract_from(&header)?;
                let metabox = meta::MetaBox::from_stream_header(&mut ex, header)?;
                let mut root_meta_box_properties = meta::extract_metabox_properties(&metabox);
                root_meta_box_properties.context_id = 0;
                file_properties.root_meta_box_properties = root_meta_box_properties;
                // TODO: meta_box_info.insert(0, extract_items(metabox, 0)) 
                metabox_map.insert(0, metabox);
            } else if box_type == "moov" {
                if movie_box.is_some() {
                    return Err(HeifError::InvalidFormat);
                }
                let mut ex = stream.extract_from(&header)?;
                movie_box = Some(MovieBox::new(&mut ex, header)?);
            } else if box_type == "mdat" || box_type == "free" || box_type == "skip" {
                println!(">>SKIPPING {}", box_type.to_string());
                stream.skip_bytes(header.body_size() as usize)?;
            } else {
                println!("unknown type {}", box_type.to_string());
                stream.skip_bytes(header.body_size() as usize)?;
            }
        }
        if ftyp.is_none() || (metabox_map.is_empty() && movie_box.is_none()) {
            return Err(HeifError::InvalidFormat);
        }
        Ok(Self {
            ftyp: ftyp.unwrap(),
            metabox_map,
        })
    }
}

#[derive(Debug, Default)]
pub struct FileInformationInternal {
    file_feature: FileFeature,
    track_properties: HashMap<u32, TrackProperties>,
    root_meta_box_properties: meta::MetaBoxProperties,
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
