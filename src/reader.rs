use std::collections::{HashMap, LinkedList};
use std::fs::File;

use crate::bbox::ftyp::FileTypeBox;
use crate::bbox::header::{BoxHeader, Header};
use crate::bbox::meta::iinf::ItemInfoEntry;
use crate::bbox::meta::iloc::ConstructionMethod;
use crate::bbox::meta::iprp::hevc::HevcConfigurationBox;
use crate::bbox::meta::iprp::ispe::ImageSpatialExtentsProperty;
use crate::bbox::meta::iprp::{DecoderConfigurationRecord, PropertyType};
use crate::bbox::meta::MetaBox;
use crate::bbox::moov::MovieBox;
use crate::bit::{BitStream, Byte4, Stream};
use crate::data::*;
use crate::internal::*;
use crate::{HeifError, Result};

#[derive(Debug)]
struct ItemInfo {
    pub item_type: Byte4,
    pub name: String,
    pub content_type: String,
    pub content_encoding: String,
    pub width: u32,
    pub height: u32,
    // pub display_time: u64,
}

type ItemInfoMap = HashMap<u32, ItemInfo>;
type Properties = HashMap<u32, PropertyTypeVector>;

#[derive(Default, Debug)]
struct MetaBoxInfo {
    pub displayable_master_images: usize,
    pub item_info_map: ItemInfoMap,
    pub grid_items: HashMap<u32, Grid>,
    pub iovl_items: HashMap<u32, Overlay>,
    pub properties: Properties,
}

#[derive(Default, Debug)]
struct SampleInfo {
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
    pub samples: SampleInfoVector,
    pub width: u32,
    pub height: u32,
    pub matrix: Vec<i32>,
    pub duration: f64,
    // TODO: pMap
    pub clap_properties: HashMap<u32, CleanAperture>,
    pub auxi_properties: HashMap<u32, AuxiliaryType>,
    pub repetitions: f64,
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Uninitialized,
    Initializing,
    Ready,
}

impl Default for State {
    fn default() -> Self {
        State::Uninitialized
    }
}

#[derive(Debug, Default)]
pub struct HeifReader {
    state: State,
    stream: BitStream,
    file_properties: FileInformationInternal,
    decoder_code_type_map: HashMap<Id, Byte4>,
    parameter_set_map: HashMap<Id, ParameterSetMap>,
    image_to_parameter_set_map: HashMap<Id, Id>,
    primary_item_id: Option<u32>,
    ftyp: FileTypeBox,
    file_information: FileInformation,
    metabox_map: HashMap<u32, MetaBox>,
    metabox_info: HashMap<u32, MetaBoxInfo>,
    matrix: Vec<i32>,
    track_info: HashMap<u32, TrackInfo>,
}

// accessors
impl HeifReader {
    fn is_initialized(&self) -> bool {
        self.state != State::Uninitialized
    }

    fn check_initialized<T>(&self, v: T) -> Result<T> {
        if !self.is_initialized() {
            Err(HeifError::Uninitialized)
        } else {
            Ok(v)
        }
    }

    pub fn file_information(&self) -> Result<&FileInformation> {
        self.check_initialized(&self.file_information)
    }

    pub fn major_brand(&self) -> Result<&Byte4> {
        self.check_initialized(&self.ftyp.major_brand())
    }

    pub fn minor_version(&self) -> Result<u32> {
        self.check_initialized(self.ftyp.minor_version())
    }

    pub fn compatible_brands(&self) -> Result<&Vec<Byte4>> {
        self.check_initialized(&self.ftyp.compatible_brands())
    }

    pub fn display_width(&self, sequence_id: u32) -> Result<u32> {
        Ok(self.get_track_by_sequence_id(sequence_id)?.width)
    }

    pub fn display_height(&self, sequence_id: u32) -> Result<u32> {
        Ok(self.get_track_by_sequence_id(sequence_id)?.height)
    }

    pub fn width(&self, item_id: u32) -> Result<u32> {
        if let Some(info) = self.root_meta_box_info()?.item_info_map.get(&item_id) {
            Ok(info.width)
        } else {
            Err(HeifError::InvalidItemID)
        }
    }

    pub fn width_at(&self, sequence_id: u32, item_id: u32) -> Result<u32> {
        let track = self.get_track_by_sequence_id(sequence_id)?;
        if let Some(sample) = track.samples.get(item_id as usize) {
            Ok(sample.width)
        } else {
            Err(HeifError::InvalidItemID)
        }
    }

    pub fn height(&self, item_id: u32) -> Result<u32> {
        let root = self.root_meta_box_info()?;
        if let Some(info) = root.item_info_map.get(&item_id) {
            Ok(info.height)
        } else {
            Err(HeifError::InvalidItemID)
        }
    }

    pub fn height_at(&self, sequence_id: u32, item_id: u32) -> Result<u32> {
        let track = self.get_track_by_sequence_id(sequence_id)?;
        if let Some(sample) = track.samples.get(item_id as usize) {
            Ok(sample.height)
        } else {
            Err(HeifError::InvalidItemID)
        }
    }

    pub fn matrix(&self) -> &Vec<i32> {
        &self.matrix
    }

    pub fn grid_item_by_id(&self, item_id: u32) -> Result<&Grid> {
        if self.is_protected(item_id)? {
            return Err(HeifError::ProtectedItem);
        }
        if let Some(g) = self.root_meta_box_info()?.grid_items.get(&item_id) {
            Ok(g)
        } else {
            Err(HeifError::InvalidItemID)
        }
    }

    pub fn get_item_list_by_type(&self, item_type: Byte4) -> Result<IdVec> {
        Ok(self
            .image_item_ids()?
            .into_iter()
            .filter(|id| {
                if let Ok(ty) = self.get_item_type(*id) {
                    *ty == item_type
                } else {
                    false
                }
            })
            .collect())
    }

    pub fn get_item_data_with_decoder_parameters(&self, item_id: u32) -> Result<Vec<Vec<u8>>> {
        let item = self.get_item_by_image_id(item_id)?;
        if item.is_protected() {
            return Err(HeifError::ProtectedItem);
        }
        let code_type = self.get_decoder_code_type(item_id)?;
        if code_type != "hvc1" && code_type != "avc1" {
            return Err(HeifError::UnsupportedCodeType);
        }
        let decoder_infos = self.get_decoder_parameter_sets(item_id)?;
        let mut data_buf = Vec::new();
        for config in decoder_infos.decoder_specific_info {
            data_buf.push(config.dec_spec_info_data);
        }
        data_buf.push(self.get_item_data(item_id, true)?);
        Ok(data_buf)
    }

    pub fn get_item_data(&self, item_id: u32, byte_stream_headers: bool) -> Result<Vec<u8>> {
        if !self.is_valid_item(item_id)? {
            return Err(HeifError::InvalidItemID);
        }
        let mut past_references = LinkedList::new();
        let root_metabox = self.root_meta_box()?;
        let item_length = self.get_item_length(root_metabox, item_id, &mut past_references)?;
        if item_length > self.stream.len() {
            return Err(HeifError::FileHeader);
        }
        let mut buf = self.read_item(root_metabox, item_id, item_length)?;
        let raw_type = match root_metabox.item_info_box().item_by_id(item_id) {
            Some(i) => i.item_type(),
            None => return Err(HeifError::InvalidItemID),
        };
        let is_protected = self.is_protected(item_id)?;
        let process_data = !is_protected && (raw_type == "hvc1" || raw_type == "avc1");
        if process_data && byte_stream_headers {
            let code_type = self.get_decoder_code_type(item_id)?;
            if code_type == "avc1" {
                unimplemented!("get_item_data avc1");
            } else if code_type == "hvc1" {
                process_hevc_item_data(&mut buf);
            } else {
                return Err(HeifError::UnsupportedCodeType);
            }
        }
        Ok(buf)
    }

    pub fn get_master_image_ids(&self) -> Result<IdVec> {
        let root_metabox = self.root_meta_box()?;
        Ok(self
            .image_item_ids()?
            .into_iter()
            .filter(|item_id| {
                let ty = self
                    .get_item_by_image_id(*item_id)
                    .unwrap()
                    .item_type()
                    .to_string();
                (ty == "avc1" || ty == "hvc1")
                    && (!do_references_from_item_id_exist(
                        root_metabox,
                        *item_id,
                        "auxl".parse().unwrap(),
                    ) && !do_references_from_item_id_exist(
                        root_metabox,
                        *item_id,
                        "thmb".parse().unwrap(),
                    ))
            })
            .collect())
    }
}

// load
impl HeifReader {
    pub fn load(&mut self, file_path: &str) -> Result<()> {
        let mut file = File::open(file_path).map_err(|_| HeifError::FileOpen)?;
        self.reset();

        self.stream = BitStream::from(&mut file)?;
        self.read_stream()?;

        self.file_information.root_meta_box_information =
            self.convert_root_meta_box_information(&self.file_properties.root_meta_box_properties)?;
        // TODO: convertTrackInformation
        self.file_information.features = self.file_properties.file_feature.feature_mask();
        self.file_information.movie_timescale = self.file_properties.movie_timescale;

        println!(">> fileProperties: {:?}\n", self.file_properties);
        println!(">> decoderCodeTypeMap: {:?}\n", self.decoder_code_type_map);
        println!(">> parameterSetMap: {:?}\n", self.parameter_set_map);
        println!(
            ">> imageToParameterSetMap: {:?}\n",
            self.image_to_parameter_set_map
        );
        println!(">> ftyp: {:?}\n", self.ftyp);
        println!(">> fileInformation: {:?}\n", self.file_information);
        println!(">> metaboxMap: {:?}\n", self.metabox_map);
        println!(">> metaboxInfo: {:?}\n", self.metabox_info);
        println!(">> trackInfo: {:?}\n", self.track_info);

        Ok(())
    }

    fn get_item_type(&self, item_id: u32) -> Result<&Byte4> {
        let root_metabox_info = self.root_meta_box_info()?;
        if let Some(info) = root_metabox_info.item_info_map.get(&item_id) {
            Ok(&info.item_type)
        } else {
            Err(HeifError::InvalidItemID)
        }
    }

    fn reset(&mut self) {
        self.state = State::Uninitialized;
        self.stream.clear();
        self.file_properties = FileInformationInternal::default();
        self.decoder_code_type_map.clear();
        self.parameter_set_map.clear();
        self.image_to_parameter_set_map.clear();
        self.primary_item_id = None;
        self.ftyp = FileTypeBox::default();
        self.file_information = FileInformation::default();
        self.metabox_map.clear();
        self.metabox_info.clear();
        self.matrix.clear();
        self.track_info.clear();
    }

    fn read_stream(&mut self) -> Result<()> {
        self.state = State::Initializing;

        let mut ftyp_found = false;
        let mut metabox_found = false;
        let mut movie_found = false;

        while !self.stream.is_eof() {
            let header = BoxHeader::from_stream(&mut self.stream)?;
            let box_type = header.box_type();
            match box_type.to_string().as_str() {
                "ftyp" => {
                    if ftyp_found {
                        return Err(HeifError::FileRead);
                    }
                    ftyp_found = true;
                    self.read_ftyp(header)?;
                }
                "meta" => {
                    if metabox_found {
                        return Err(HeifError::FileRead);
                    }
                    metabox_found = true;
                    self.read_meta(header)?;
                }
                "moov" => {
                    if movie_found {
                        return Err(HeifError::FileRead);
                    }
                    movie_found = true;
                    self.read_moov(header)?;
                }
                "mdat" | "free" | "skip" => {
                    self.stream.skip_bytes(header.body_size() as usize)?;
                }
                _ => {
                    println!("unknown type {}", box_type.to_string());
                    self.stream.skip_bytes(header.body_size() as usize)?;
                }
            }
        }
        if !ftyp_found || (!metabox_found && !movie_found) {
            return Err(HeifError::FileHeader);
        }
        self.state = State::Ready;
        // TODO: file_features
        Ok(())
    }

    fn read_ftyp(&mut self, header: BoxHeader) -> Result<()> {
        let mut ex = self.stream.extract_from(&header)?;
        self.ftyp = FileTypeBox::new(&mut ex, header)?;
        Ok(())
    }

    fn read_meta(&mut self, header: BoxHeader) -> Result<()> {
        let mut ex = self.stream.extract_from(&header)?;
        let context_id = 0;
        self.metabox_map
            .insert(context_id, MetaBox::from_stream_header(&mut ex, header)?);
        let metabox = &self.metabox_map[&context_id];

        self.file_properties.root_meta_box_properties = extract_metabox_properties(&metabox);
        self.metabox_info
            .insert(context_id, self.extract_items(&metabox, context_id)?);

        self.process_decoder_config_properties(context_id);

        let master_image_ids = self.get_master_image_ids()?;
        if let Some(m) = self.metabox_info.get_mut(&context_id) {
            m.displayable_master_images = master_image_ids.len();
        }

        for (id, item_feature) in &self
            .file_properties
            .root_meta_box_properties
            .item_features_map
        {
            if item_feature.has_feature(ItemFeatureEnum::IsPrimaryImage) {
                self.primary_item_id = Some(*id);
            }
        }
        Ok(())
    }

    fn read_moov(&mut self, header: BoxHeader) -> Result<()> {
        let mut ex = self.stream.extract_from(&header)?;
        let movie_box = MovieBox::new(&mut ex, header)?;
        Ok(())
    }

    fn extract_items(&self, metabox: &MetaBox, context_id: u32) -> Result<MetaBoxInfo> {
        let mut metabox_info = MetaBoxInfo::default();
        for item in metabox.item_info_box().item_info_list() {
            let item_type = item.item_type();
            if item_type == "grid" || item_type == "iovl" {
                if item.is_protected() {
                    continue;
                }
                let mut ex_stream = self.load_item_data(metabox, item.item_id())?;
                if item_type == "grid" {
                    let image_grid = parse_image_grid(&mut ex_stream)?;
                    metabox_info.grid_items.insert(
                        item.item_id(),
                        Grid {
                            columns: u32::from(image_grid.columns_minus_one) + 1,
                            rows: u32::from(image_grid.rows_minus_one) + 1,
                            output_width: image_grid.output_width,
                            output_height: image_grid.output_height,
                            image_ids: self.get_referenced_from_item_list_by_type(
                                item.item_id(),
                                "dimg".parse().unwrap(),
                            )?,
                        },
                    );
                } else {
                    // iovl
                    unimplemented!("extract_items iovl {:?}", ex_stream);
                }
            }
        }
        metabox_info.properties = self.process_item_properties(context_id)?;
        metabox_info.item_info_map = extract_item_info_map(metabox);
        Ok(metabox_info)
    }

    fn get_referenced_from_item_list_by_type(
        &self,
        from_item_id: u32,
        reference_type: Byte4,
    ) -> Result<Vec<u32>> {
        self.get_item_by_image_id(from_item_id)?;
        let item_reference_box = self.root_meta_box()?.item_reference_box();
        let references = item_reference_box.references_of_type(reference_type);
        let mut item_id_vec = IdVec::new();
        for reference in references {
            if reference.get_from_item_id() == from_item_id {
                let mut to_ids = reference.to_item_ids().clone();
                item_id_vec.append(&mut to_ids);
            }
        }
        Ok(item_id_vec)
    }

    fn load_item_data(&self, metabox: &MetaBox, item_id: u32) -> Result<BitStream> {
        let mut past_references = LinkedList::new();
        let item_length = self.get_item_length(metabox, item_id, &mut past_references)?;
        if !self.stream.has_bytes(item_length) {
            return Err(HeifError::FileHeader);
        }
        Ok(BitStream::new(self.read_item(
            metabox,
            item_id,
            item_length,
        )?))
    }

    fn read_item(&self, metabox: &MetaBox, item_id: u32, max_size: usize) -> Result<Vec<u8>> {
        if !self.is_valid_item(item_id)? {
            return Err(HeifError::InvalidItemID);
        }

        let iloc = metabox.item_location_box();
        let version = iloc.full_box_header().version();
        let item_location = match iloc.item_location_by_id(item_id) {
            Some(i) => i,
            None => return Err(HeifError::InvalidItemID),
        };

        let construction_method = item_location.construction_method();
        let extent_list = item_location.extent_list();
        let base_offset = item_location.base_offset();
        if extent_list.is_empty() {
            return Err(HeifError::FileRead);
        }

        let mut res = Vec::with_capacity(max_size);
        let mut total_length = 0;
        if version == 0 || (version >= 1 && construction_method == ConstructionMethod::FileOffset) {
            for extent in extent_list {
                let offset = base_offset + extent.extent_offset;
                if total_length + extent.extent_length > max_size {
                    return Err(HeifError::FileRead);
                }
                let slice = self.stream.slice(offset, extent.extent_length)?;
                res.append(&mut slice.to_vec());
                total_length += extent.extent_length;
            }
        } else if version >= 1 && (construction_method == ConstructionMethod::IdatOffset) {
            for extent in extent_list {
                let offset = base_offset + extent.extent_offset;
                if total_length + extent.extent_length > max_size {
                    return Err(HeifError::FileRead);
                }
                let mut memory_buf =
                    match metabox.item_data_box().read(offset, extent.extent_length) {
                        Some(m) => m,
                        None => return Err(HeifError::FileRead),
                    };
                res.append(&mut memory_buf.to_vec());
                total_length += extent.extent_length;
            }
        } else if version >= 1 && (construction_method == ConstructionMethod::ItemOffset) {
            unimplemented!("itemoffset")
        } else {
            return Err(HeifError::FileRead);
        }
        Ok(res)
    }

    fn process_item_properties(&self, context_id: u32) -> Result<Properties> {
        let mut propety_map = Properties::default();
        if let Some(metabox) = self.metabox_map.get(&context_id) {
            let iprp = metabox.item_properties_box();
            for item_id in metabox.item_info_box().item_ids() {
                propety_map.insert(
                    item_id,
                    iprp.get_item_properties(item_id)?
                        .iter()
                        .map(|prop| ItemPropertyInfo {
                            item_property_type: ItemPropertyType::from(prop.property_type),
                            index: prop.index,
                            is_essential: prop.is_essential,
                        })
                        .collect(),
                );
            }
        }
        Ok(propety_map)
    }

    fn get_decoder_code_type(&self, item_id: u32) -> Result<&Byte4> {
        if !self.is_valid_image_item(item_id)? {
            return Err(HeifError::InvalidItemID);
        }
        match self.decoder_code_type_map.get(&(
            self.file_properties.root_meta_box_properties.context_id,
            item_id,
        )) {
            Some(ty) => Ok(ty),
            None => Err(HeifError::InvalidItemID),
        }
    }

    fn get_decoder_parameter_sets(&self, item_id: u32) -> Result<DecoderConfiguration> {
        if !self.is_valid_image_item(item_id)? {
            return Err(HeifError::InvalidItemID);
        }
        let image_full_id = (
            self.file_properties.root_meta_box_properties.context_id,
            item_id,
        );
        let parameter_set_id = match self.image_to_parameter_set_map.get(&image_full_id) {
            Some(id) => id,
            None => return Err(HeifError::InvalidItemID),
        };
        let parameter_set_map = match self.parameter_set_map.get(&parameter_set_id) {
            Some(m) => m,
            None => return Err(HeifError::FileHeader),
        };
        let decoder_config_id = parameter_set_id.1;
        let decoder_specific_info = parameter_set_map
            .iter()
            .map(
                |(dec_spec_info_type, dec_spec_info_data)| DecoderSpecificInfo {
                    dec_spec_info_type: *dec_spec_info_type,
                    dec_spec_info_data: dec_spec_info_data.to_vec(),
                },
            )
            .collect();
        Ok(DecoderConfiguration {
            decoder_config_id,
            decoder_specific_info,
        })
    }

    fn process_decoder_config_properties(&mut self, context_id: u32) {
        if let Some(iprp) = self.metabox_map.get(&context_id) {
            let iprp = iprp.item_properties_box();
            for image_properties in &self
                .file_properties
                .root_meta_box_properties
                .item_features_map
            {
                let image_id = *image_properties.0;
                let id: Id = (context_id, image_id);
                let hvcc_index = iprp.find_property_index(PropertyType::HVCC, image_id);
                let avcc_index = iprp.find_property_index(PropertyType::AVCC, image_id);
                let mut config_index: Id;
                if hvcc_index != 0 {
                    config_index = (context_id, hvcc_index);
                } else if avcc_index != 0 {
                    // TODO
                    unimplemented!("avcc_index");
                } else {
                    continue;
                }
                if let Some(prop) = iprp.property_by_index(config_index.1 as usize - 1) {
                    if let Some(hevc_box) = prop.as_any().downcast_ref::<HevcConfigurationBox>() {
                        config_index = (context_id, hvcc_index);
                        if self.parameter_set_map.get(&config_index).is_none() {
                            self.parameter_set_map.insert(
                                config_index,
                                self.make_decoder_parameter_set_map(hevc_box.config()),
                            );
                        }
                        self.image_to_parameter_set_map.insert(id, config_index);
                        self.decoder_code_type_map
                            .insert(id, "hvc1".parse().unwrap());
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
            let spec_info_type = DecoderSpecInfoType::from(k);
            if let Some(m) = pm.get_mut(&spec_info_type) {
                let mut tmp = v.clone();
                tmp.append(m);
                *m = tmp;
            } else {
                pm.insert(spec_info_type, v);
            }
        }
        pm
    }

    fn convert_root_meta_box_information(
        &self,
        metabox_properties: &MetaBoxProperties,
    ) -> Result<MetaBoxInformation> {
        let mut item_informations = Vec::new();
        for (id, item) in &metabox_properties.item_features_map {
            let root_meta_info = self.root_meta_box_info()?;
            if let Some(item_info) = root_meta_info.item_info_map.get(id) {
                let mut past_references = LinkedList::new();
                let root_metabox = self.root_meta_box().unwrap();
                let size = self
                    .get_item_length(root_metabox, *id, &mut past_references)
                    .unwrap_or(0);
                item_informations.push(ItemInformation {
                    item_id: *id,
                    item_type: item_info.item_type.clone(),
                    description: ItemDescription {
                        name: item_info.name.clone(),
                        content_type: item_info.content_type.clone(),
                        content_encoding: item_info.content_encoding.clone(),
                    },
                    features: item.feature_mask(),
                    size,
                });
            }
        }

        Ok(MetaBoxInformation {
            features: metabox_properties.meta_box_feature.feature_mask(),
            item_informations,
            entity_groupings: metabox_properties.entity_groupings.clone(),
        })
    }

    fn get_item_length(
        &self,
        metabox: &MetaBox,
        item_id: u32,
        past_references: &mut LinkedList<u32>,
    ) -> Result<usize> {
        if !self.is_valid_item(item_id)? {
            return Err(HeifError::InvalidItemID);
        }

        let mut found = false;
        for i in past_references.iter_mut() {
            if *i == item_id {
                found = true;
                break;
            }
        }
        if !found {
            past_references.push_back(item_id);
        } else {
            return Err(HeifError::FileHeader);
        }

        let iloc = metabox.item_location_box();
        let version = iloc.full_box_header().version();
        let item_location = match iloc.item_location_by_id(item_id) {
            Some(i) => i,
            None => return Err(HeifError::InvalidItemID),
        };
        let extent_list = item_location.extent_list();
        if extent_list.is_empty() {
            return Err(HeifError::FileRead);
        }
        let mut item_length = 0;
        if version >= 1 && (item_location.construction_method() == ConstructionMethod::ItemOffset) {
            let all_iloc_references = metabox
                .item_reference_box()
                .references_of_type("iloc".parse().unwrap());
            let to_item_ids = match all_iloc_references
                .iter()
                .find(|item| item.get_from_item_id() == item_id)
            {
                Some(iloc) => iloc.to_item_ids(),
                None => return Err(HeifError::FileRead),
            };
            for extent in extent_list {
                let extent_source_item_index = if iloc.index_size() != 0 {
                    extent.extent_index
                } else {
                    1
                };
                let sub_item_id = match to_item_ids.get(extent_source_item_index - 1) {
                    Some(i) => i,
                    None => return Err(HeifError::FileHeader),
                };
                if item_id == *sub_item_id {
                    return Err(HeifError::FileHeader);
                }
                let sub_item_length =
                    self.get_item_length(metabox, *sub_item_id, past_references)?;
                item_length += if extent.extent_length == 0 {
                    sub_item_length as usize
                } else {
                    extent.extent_length
                };
            }
        } else {
            for extent in extent_list {
                item_length += extent.extent_length;
            }
        }
        Ok(item_length)
    }
}

impl HeifReader {
    // Root MetaBox and MetaBoxInfo

    fn root_meta_box(&self) -> Result<&MetaBox> {
        self.check_initialized(
            &self.metabox_map[&self.file_properties.root_meta_box_properties.context_id],
        )
    }

    fn root_meta_box_info(&self) -> Result<&MetaBoxInfo> {
        self.check_initialized(
            &self.metabox_info[&self.file_properties.root_meta_box_properties.context_id],
        )
    }

    // Item

    fn get_item_by_image_id(&self, image_id: u32) -> Result<&ItemInfoEntry> {
        if !self.is_initialized() {
            return Err(HeifError::Uninitialized);
        }
        match self.root_meta_box()?.item_info_box().item_by_id(image_id) {
            Some(item) => Ok(item),
            None => Err(HeifError::InvalidItemID),
        }
    }

    fn is_valid_item(&self, image_id: u32) -> Result<bool> {
        self.check_initialized(
            self.root_meta_box()?
                .item_info_box()
                .item_by_id(image_id)
                .is_some(),
        )
    }

    fn is_valid_image_item(&self, image_id: u32) -> Result<bool> {
        let item_info_entry = self.get_item_by_image_id(image_id)?;
        Ok(is_image_item_type(item_info_entry.item_type()))
    }

    fn is_protected(&self, item_id: u32) -> Result<bool> {
        Ok(self.get_item_by_image_id(item_id)?.is_protected())
    }

    fn image_item_ids(&self) -> Result<IdVec> {
        Ok(self
            .root_meta_box()?
            .item_info_box()
            .item_info_list()
            .iter()
            .filter(|item_info_entry| is_image_item_type(item_info_entry.item_type()))
            .map(ItemInfoEntry::item_id)
            .collect())
    }

    // Track

    fn get_track_by_sequence_id(&self, sequence_id: u32) -> Result<&TrackInfo> {
        if !self.is_initialized() {
            return Err(HeifError::Uninitialized);
        }
        match self.track_info.get(&sequence_id) {
            Some(t) => Ok(t),
            None => Err(HeifError::InvalidSequenceID),
        }
    }

    fn get_sequence_items(&self, sequence_id: u32) -> Result<IdVec> {
        Ok(self
            .get_track_by_sequence_id(sequence_id)?
            .samples
            .iter()
            .map(|s| s.decoding_order)
            .collect())
    }
}

// internal
fn parse_image_grid(stream: &mut BitStream) -> Result<ImageGrid> {
    stream.read_byte()?;
    let read_4bytes_fields = (stream.read_byte()? & 1) != 0;
    let rows_minus_one = stream.read_byte()?;
    let columns_minus_one = stream.read_byte()?;
    let (output_width, output_height) = if read_4bytes_fields {
        (
            stream.read_4bytes()?.to_u32(),
            stream.read_4bytes()?.to_u32(),
        )
    } else {
        (
            stream.read_2bytes()?.to_u32(),
            stream.read_2bytes()?.to_u32(),
        )
    };
    Ok(ImageGrid {
        rows_minus_one,
        columns_minus_one,
        output_width,
        output_height,
    })
}

fn do_references_from_item_id_exist(metabox: &MetaBox, item_id: u32, ref_type: Byte4) -> bool {
    metabox
        .item_reference_box()
        .references_of_type(ref_type)
        .iter()
        .any(|r| r.get_from_item_id() == item_id)
}

fn is_image_item_type(item_type: &Byte4) -> bool {
    let item_type = item_type.to_string();
    item_type == "avc1"
        || item_type == "hvc1"
        || item_type == "grid"
        || item_type == "iovl"
        || item_type == "iden"
        || item_type == "jpeg"
}

fn extract_metabox_item_properties_map(metabox: &MetaBox) -> HashMap<u32, ItemFeature> {
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
            if item.item_protection_index() > 0 {
                item_features.set_feature(ItemFeatureEnum::IsProtected);
            }
            if do_references_from_item_id_exist(metabox, item_id, "thmb".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::IsThumbnailImage);
            }
            if do_references_from_item_id_exist(metabox, item_id, "auxl".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::IsAuxiliaryImage);
            }
            if do_references_from_item_id_exist(metabox, item_id, "base".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::IsPreComputedDerivedImage);
            }
            if do_references_from_item_id_exist(metabox, item_id, "dimg".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::IsDerivedImage);
            }
            if !item_features.has_feature(ItemFeatureEnum::IsThumbnailImage)
                && !item_features.has_feature(ItemFeatureEnum::IsAuxiliaryImage)
            {
                item_features.set_feature(ItemFeatureEnum::IsMasterImage);
            }
            if do_references_from_item_id_exist(metabox, item_id, "thmb".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::HasLinkedThumbnails);
            }
            if do_references_from_item_id_exist(metabox, item_id, "auxl".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::HasLinkedAuxiliaryImage);
            }
            if do_references_from_item_id_exist(metabox, item_id, "cdsc".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::HasLinkedMetadata);
            }
            if do_references_from_item_id_exist(metabox, item_id, "base".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::HasLinkedPreComputedDerivedImage);
            }
            if do_references_from_item_id_exist(metabox, item_id, "tbas".parse().unwrap()) {
                item_features.set_feature(ItemFeatureEnum::HasLinkedTiles);
            }
            if do_references_from_item_id_exist(metabox, item_id, "dimg".parse().unwrap()) {
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
            if do_references_from_item_id_exist(metabox, item_id, "cdsc".parse().unwrap()) {
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

fn extract_metabox_entity_to_group_maps(metabox: &MetaBox) -> Groupings {
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

fn extract_metabox_feature(
    image_features: &ItemFeaturesMap,
    groupings: Groupings,
) -> MetaBoxFeature {
    let mut meta_box_feature = MetaBoxFeature::default();
    if !groupings.is_empty() {
        meta_box_feature.set_feature(MetaBoxFeatureEnum::HasGroupLists);
    }
    if image_features.len() == 1 {
        meta_box_feature.set_feature(MetaBoxFeatureEnum::IsSingleImage);
    } else if image_features.len() > 1 {
        meta_box_feature.set_feature(MetaBoxFeatureEnum::IsImageCollection);
    }

    for i in image_features {
        let features = i.1;
        if features.has_feature(ItemFeatureEnum::IsMasterImage) {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::HasMasterImages);
        }
        if features.has_feature(ItemFeatureEnum::IsThumbnailImage) {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::HasThumbnails);
        }
        if features.has_feature(ItemFeatureEnum::IsAuxiliaryImage) {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::HasAuxiliaryImages);
        }
        if features.has_feature(ItemFeatureEnum::IsDerivedImage) {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::HasDerivedImages);
        }
        if features.has_feature(ItemFeatureEnum::IsPreComputedDerivedImage) {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::HasPreComputedDerivedImages);
        }
        if features.has_feature(ItemFeatureEnum::IsHiddenImage) {
            meta_box_feature.set_feature(MetaBoxFeatureEnum::HasHiddenImages);
        }
    }
    meta_box_feature
}

fn extract_metabox_properties(metabox: &MetaBox) -> MetaBoxProperties {
    let item_features_map = extract_metabox_item_properties_map(metabox);
    let entity_groupings = extract_metabox_entity_to_group_maps(metabox);
    let meta_box_feature = extract_metabox_feature(&item_features_map, entity_groupings.clone());
    MetaBoxProperties {
        context_id: 0,
        meta_box_feature,
        item_features_map,
        entity_groupings,
    }
}

fn extract_item_info_map(metabox: &MetaBox) -> ItemInfoMap {
    let mut item_info_map = ItemInfoMap::new();
    for item_id in metabox.item_info_box().item_ids() {
        let item = metabox.item_info_box().item_by_id(item_id).unwrap();

        let item_type = item.item_type().clone();
        let name = item.item_name().clone();
        let content_type = item.content_type().clone();
        let content_encoding = item.content_encoding().clone();
        let mut width = 0;
        let mut height = 0;
        if is_image_item_type(&item_type) {
            let iprp = metabox.item_properties_box();
            let ispe_index = iprp.find_property_index(PropertyType::ISPE, item_id);
            if ispe_index != 0 {
                if let Some(b) = iprp.property_by_index(ispe_index as usize - 1) {
                    if let Some(image_spatial_extents_properties) =
                        b.as_any().downcast_ref::<ImageSpatialExtentsProperty>()
                    {
                        width = image_spatial_extents_properties.width();
                        height = image_spatial_extents_properties.height();
                    }
                }
            }
        }
        item_info_map.insert(
            item_id,
            ItemInfo {
                item_type,
                name,
                content_type,
                content_encoding,
                width,
                height,
            },
        );
    }
    item_info_map
}

fn process_hevc_item_data(buf: &mut Vec<u8>) {
    let mut output_offset = 0;
    let mut byte_offset = 0;
    let len = buf.len();
    while output_offset < len {
        let mut nal_length = u32::from(buf[output_offset + byte_offset]);
        buf[output_offset + byte_offset] = 0;
        byte_offset += 1;
        nal_length = (nal_length << 8) | u32::from(buf[output_offset + byte_offset]);
        buf[output_offset + byte_offset] = 0;
        byte_offset += 1;
        nal_length = (nal_length << 8) | u32::from(buf[output_offset + byte_offset]);
        buf[output_offset + byte_offset] = 0;
        byte_offset += 1;
        nal_length = (nal_length << 8) | u32::from(buf[output_offset + byte_offset]);
        buf[output_offset + byte_offset] = 1;
        output_offset += (nal_length + 4) as usize;
        byte_offset = 0;
    }
}
