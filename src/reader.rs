use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::Result;
use crate::_box::ftyp::FileTypeBox;
use crate::_box::meta::MetaBox;
use crate::_box::moov::MovieBox;
use crate::_box::{BoxHeader, Header};
use crate::bit::{BitStream, Stream};
use crate::error::HeifError;

pub struct HeifReader {
    ftyp: FileTypeBox,
    metabox_map: HashMap<u32, MetaBox>,
    //file_properties: FileInformation,
}

impl HeifReader {
    pub fn from(file_path: &str) -> Result<HeifReader> {
        let mut file = File::open(file_path).map_err(|_| HeifError::FileOpen)?;
        let mut stream = BitStream::from(&mut file)?;
        let mut ftyp = Option::<FileTypeBox>::None;
        let mut metabox_map = HashMap::new();
        let mut movie_box = Option::<MovieBox>::None;

        while !stream.is_eof() {
            let header = BoxHeader::new(&mut stream)?;
            if header.box_type == "ftyp" {
                if ftyp.is_some() {
                    return Err(HeifError::InvalidFormat);
                }
                let mut ex = stream.extract_from(&header)?;
                ftyp = Some(FileTypeBox::new(&mut ex, header)?);
            } else if header.box_type == "meta" {
                if metabox_map.get(&0).is_some() {
                    return Err(HeifError::InvalidFormat);
                }
                let mut ex = stream.extract_from(&header)?;
                metabox_map.insert(0, MetaBox::new(&mut ex, header)?);
            } else if header.box_type == "moov" {
                if movie_box.is_some() {
                    return Err(HeifError::InvalidFormat);
                }
                let mut ex = stream.extract_from(&header)?;
                movie_box = Some(MovieBox::new(&mut ex, header)?);
            } else if header.box_type == "mdat"
                || header.box_type == "free"
                || header.box_type == "skip"
            {
                println!(">>SKIPPING {:?}", header);
                stream.skip_bytes(header.body_size() as usize)?;
            } else {
                println!("skipping unknown type {}", header.box_type);
                stream.skip_bytes(header.body_size() as usize)?;
            }
        }
        if ftyp.is_none() || (metabox_map.get(&0).is_none() && movie_box.is_none()) {
            return Err(HeifError::InvalidFormat);
        }
        let ftyp = ftyp.unwrap();
        Ok(HeifReader { ftyp, metabox_map })
    }
}

#[derive(Debug)]
struct FileInformation {
    file_feature: FileFeature,
    track_properties: HashMap<u32, TrackProperties>,
    root_meta_box_properties: MetaBoxProperties,
    movie_timescale: u32,
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum FileFeatureEnum {}

#[derive(Debug)]
struct FileFeature {
    file_feature_set: HashSet<FileFeatureEnum>,
}

type IdVec = Vec<u32>;

#[derive(Debug)]
struct TrackProperties {
    track_id: u32,
    alternate_group_id: u32,
    track_feature: TrackFeature,
    sample_properties: SampleProperties,
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

#[derive(Debug)]
struct TrackFeature {}

#[derive(Debug)]
struct SampleProperties {}

#[derive(Debug)]
struct SampleGrouping {}

#[derive(Debug)]
struct SampleToMetadataItem {}

#[derive(Debug)]
struct SampleVisualEquivalence {}

#[derive(Debug)]
struct DirectReferenceSamples {}

#[derive(Debug)]
struct EditList {}

#[derive(Debug)]
struct MetaBoxProperties {
    context_id: u32,
    meta_box_feature: MetaBoxFeature,
    item_features_map: HashMap<u32, ItemFeature>,
    entity_groupings: Vec<EntityGrouping>,
}

#[derive(Debug)]
struct MetaBoxFeature {}

#[derive(Debug)]
struct ItemFeature {}

#[derive(Debug)]
struct EntityGrouping {
    group_type: String,
    group_id: u32,
    entity_ids: Vec<u32>,
}
