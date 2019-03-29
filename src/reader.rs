use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;

use crate::bbox::ftyp::FileTypeBox;
use crate::bbox::header::{BoxHeader, Header};
use crate::bbox::meta::iprp::hevc::HevcConfigurationBox;
use crate::bbox::meta::iprp::{DecoderConfigurationRecord, DecoderParameterType, PropertyType};
use crate::bbox::meta::MetaBox;
use crate::bbox::moov::MovieBox;
use crate::bit::{BitStream, Byte4, Stream};
use crate::data::*;
use crate::internal::*;
use crate::{HeifError, Result};

#[derive(Debug, Default)]
pub struct HeifReader {
    file_properties: FileInformationInternal,
    decoder_code_type_map: HashMap<Id, Byte4>,
    parameter_set_map: HashMap<Id, ParameterSetMap>,
    image_to_parameter_set_map: HashMap<Id, Id>,
    // primary_item_id: Option<u32>,
    ftyp: FileTypeBox,
    // file_information: FileInformation,
    metabox_map: HashMap<u32, MetaBox>,
    metabox_info: HashMap<u32, MetaBoxInfo>,
    // matrix: Vec<i32>,
    // track_info: HashMap<u32, TrackInfo>,
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
                let context_id = 0;
                let mut ex = stream.extract_from(&header)?;
                self.metabox_map
                    .insert(context_id, MetaBox::from_stream_header(&mut ex, header)?);
                let metabox = self.metabox_map.get(&context_id).unwrap();
                self.file_properties.root_meta_box_properties =
                    self.extract_metabox_properties(&metabox);
                self.metabox_info
                    .insert(context_id, self.extract_items(&metabox, context_id));

                self.process_decoder_config_properties(&context_id);
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

    fn extract_metabox_entity_to_group_maps(&self, metabox: &MetaBox) -> Groupings {
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
        groupings: &Groupings,
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

    pub fn process_decoder_config_properties(&mut self, context_id: &u32) {
        if let Some(iprp) = self.metabox_map.get(context_id) {
            let iprp = iprp.item_properties_box();
            for image_properties in &self
                .file_properties
                .root_meta_box_properties
                .item_features_map
            {
                let image_id = image_properties.0;
                let id: Id = (*context_id, *image_id);
                let hvcc_index = iprp.find_property_index(PropertyType::HVCC, &image_id);
                let avcc_index = iprp.find_property_index(PropertyType::AVCC, &image_id);
                let mut config_index: Id = (0, 0);
                if hvcc_index != 0 {
                    config_index = (*context_id, hvcc_index);
                } else if avcc_index != 0 {
                    unimplemented!("avcc_index");
                } else {
                    continue;
                }
                if let Some(prop) = iprp.property_by_index(config_index.1 as usize - 1) {
                    if let Some(hevc_box) = prop.as_any().downcast_ref::<HevcConfigurationBox>() {
                        let config_index: Id = (*context_id, hvcc_index);
                        if self.parameter_set_map.get(&config_index).is_none() {
                            self.parameter_set_map.insert(
                                config_index,
                                self.make_decoder_parameter_set_map(hevc_box.config()),
                            );
                        }
                        self.image_to_parameter_set_map.insert(id, config_index);
                        self.decoder_code_type_map
                            .insert(id, Byte4::from_str("hvc1").unwrap());
                    }
                }
            }
        }
    }

    fn make_decoder_parameter_set_map<T: DecoderConfigurationRecord>(
        &self,
        record: &T,
    ) -> ParameterSetMap {
        let mut pm = ParameterSetMap::default();
        for (k, v) in record.configuration_map().into_iter() {
            let ty = match k {
                DecoderParameterType::AvcSPS => DecoderSpecInfoType::AvcSPS,
                DecoderParameterType::AvcPPS => DecoderSpecInfoType::AvcPPS,
                DecoderParameterType::HevcVPS => DecoderSpecInfoType::HevcVPS,
                DecoderParameterType::HevcSPS => DecoderSpecInfoType::HevcSPS,
                DecoderParameterType::HevcPPS => DecoderSpecInfoType::HevcPPS,
                DecoderParameterType::AudioSpecificConfig => {
                    DecoderSpecInfoType::AudioSpecificConfig
                }
            };
            if let Some(m) = pm.get_mut(&ty) {
                let mut tmp = v.clone();
                tmp.append(m);
                *m = tmp;
            } else {
                pm.insert(ty, v);
            }
        }
        pm
    }
}

type Properties = HashMap<u32, PropertyTypeVector>;

#[derive(Debug)]
pub struct ItemInfo {
    pub item_type: Byte4,
    pub name: String,
    pub content_type: String,
    pub content_encoding: String,
    pub width: u32,
    pub height: u32,
    pub display_time: u64,
}

type ItemInfoMap = HashMap<u32, ItemInfo>;

#[derive(Default, Debug)]
pub struct MetaBoxInfo {
    pub displayable_master_images: usize,
    pub item_info_map: ItemInfoMap,
    pub grid_items: HashMap<u32, Grid>,
    pub iovl_items: HashMap<u32, Overlay>,
    pub properties: HashMap<u32, PropertyTypeVector>,
}

#[derive(Default, Debug)]
pub struct SampleInfo {
    decoding_order: u32,
    composition_times: Vec<i64>,
    data_offset: u64,
    data_length: u64,
    width: u32,
    height: u32,
    decode_dependencies: IdVec,
}

type SampleInfoVector = Vec<SampleInfo>;

#[derive(Debug)]
struct TrackInfo {
    samples: SampleInfoVector,
    width: u32,
    height: u32,
    matrix: Vec<i32>,
    duration: f64,
    // TODO: pMap
    clap_properties: HashMap<u32, CleanAperture>,
    auxi_properties: HashMap<u32, AuxiliaryType>,
    repetitions: f64,
}
