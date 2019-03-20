use crate::bbox::BoxHeader;
use crate::bit::BitStream;
use std::io::Result;

#[derive(Debug)]
pub struct ItemPropertiesBox {
    pub box_header: BoxHeader,
    pub container: ItemPropertyContainer,
}

impl ItemPropertiesBox {
    pub fn new(stream: &mut BitStream, box_header: BoxHeader) -> Result<Self> {
        let mut left = box_header.box_size - u64::from(box_header.header_size());
        let container_box_header = BoxHeader::new(stream)?;
        let container = ItemPropertyContainer::new(stream, container_box_header)?;
        while left > 0 {
            let sub_box_header = BoxHeader::new(stream)?;
            if sub_box_header.box_type != "ipma" {
                panic!("ipma")
            }
            unimplemented!("ItemPropertiesBox")
        }
        Ok(Self {
            box_header,
            container,
        })
    }
}

trait ItemProperty {}

#[derive(Debug)]
pub struct HevcConfigurationBox {
    pub box_header: BoxHeader,
    pub hevc_config: HevcDecoderConfigurationRecord,
}

impl HevcConfigurationBox {
    fn new(stream: &mut BitStream, box_header: BoxHeader) -> Result<Self> {
        Ok(Self {
            box_header,
            hevc_config: HevcDecoderConfigurationRecord::new(stream)?,
        })
    }
}

impl ItemProperty for HevcConfigurationBox {}

#[derive(Debug)]
pub struct HevcDecoderConfigurationRecord {
    configuration_version: u8,
    general_profile_space: u8,
    general_tier_flag: u8,
    general_profile_idc: u8,
    general_profile_compatibility_flags: u32,
    general_constraint_indicator_flags: [u8; 6],
    general_level_idc: u8,
    min_spatial_segmentation_idc: u16,
    parallelism_type: u8,
    chroma_format: u8,
    pic_width_in_luma_samples: u16,
    pic_height_in_luma_samples: u16,
    conf_win_left_offset: u16,
    conf_win_right_offset: u16,
    conf_win_top_offset: u16,
    conf_win_bottom_offset: u16,
    bit_depth_luma_minus8: u8,
    bit_depth_chroma_minus8: u8,
    avg_frame_rate: u16,
    constant_frame_rate: u8,
    num_temporal_layers: u8,
    temporal_id_nested: u8,
    length_size_minus1: u8,
    nal_array: Vec<HevcNALArray>,
}

impl HevcDecoderConfigurationRecord {
    fn new(stream: &mut BitStream) -> Result<Self> {
        let configuration_version = stream.read_byte()?;
        let general_profile_space = stream.read_bits(2)? as u8;
        let general_tier_flag = stream.read_bits(1)? as u8;
        let general_profile_idc = stream.read_bits(5)? as u8;
        let general_profile_compatibility_flags = stream.read_4bytes()?.to_u32();
        let mut general_constraint_indicator_flags = [0u8; 6];
        for i in 0..6 {
            general_constraint_indicator_flags[i] = stream.read_byte()?;
        }
        let general_level_idc = stream.read_byte()?;
        stream.read_bits(4);
        let min_spatial_segmentation_idc = stream.read_bits(12)? as u16;
        stream.read_bits(6);
        let parallelism_type = stream.read_bits(2)? as u8;
        stream.read_bits(6);
        let chroma_format = stream.read_bits(2)? as u8;
        stream.read_bits(5);
        let bit_depth_luma_minus8 = stream.read_bits(3)? as u8;
        stream.read_bits(5);
        let bit_depth_chroma_minus8 = stream.read_bits(3)? as u8;
        let avg_frame_rate = stream.read_2bytes()?.to_u16();
        let constant_frame_rate = stream.read_bits(2)? as u8;
        let num_temporal_layers = stream.read_bits(3)? as u8;
        let temporal_id_nested = stream.read_bits(1)? as u8;
        let length_size_minus1 = stream.read_bits(2)? as u8;

        let mut res = Self {
            configuration_version,
            general_profile_space,
            general_tier_flag,
            general_profile_idc,
            general_profile_compatibility_flags,
            general_constraint_indicator_flags,
            general_level_idc,
            min_spatial_segmentation_idc,
            parallelism_type,
            chroma_format,
            pic_width_in_luma_samples: 0,
            pic_height_in_luma_samples: 0,
            conf_win_left_offset: 0,
            conf_win_right_offset: 0,
            conf_win_top_offset: 0,
            conf_win_bottom_offset: 0,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            avg_frame_rate,
            constant_frame_rate,
            num_temporal_layers,
            temporal_id_nested,
            length_size_minus1,
            nal_array: Vec::new(),
        };
        let num_arrays = stream.read_byte()?;
        for _ in 0..num_arrays {
            let array_completeness = stream.read_bits(1)? != 0;
            stream.read_bits(1);
            let nal_unit_type = NalUnitType::from_u8(stream.read_bits(6)? as u8);
            let num_nalus = stream.read_2bytes()?.to_u16();
            for _ in 0..num_nalus {
                let nal_size = stream.read_2bytes()?.to_u16();
                let mut nal_data = Vec::new();
                for _ in 0..nal_size {
                    nal_data.push(stream.read_byte()?);
                }
                res.add_nal_unit(&nal_unit_type, array_completeness);
            }
        }
        Ok(res)
    }

    fn add_nal_unit(&mut self, nal_unit_type: &NalUnitType, array_completeness: bool) {
        let nal_array = self.nal_array.iter().find(|nal| nal.nal_unit_type == *nal_unit_type);
        if nal_array.is_none() {
            nal_array = Some(HevcNALArray{
                array_completeness,
                nal_unit_type,
            });
            self.nal_array.push(nal_array.unwrap());
        }
        panic!("")
    }
}

#[derive(Debug,PartialEq)]
enum NalUnitType {
    CodedSliceTrailN, // 0
    CodedSliceTrailR, // 1

    CodedSliceTsaN, // 2
    CodedSliceTsaR, // 3

    CodedSliceStsaN, // 4
    CodedSliceStsaR, // 5

    CodedSliceRadlN, // 6
    CodedSliceRadlR, // 7

    CodedSliceRaslN, // 8
    CodedSliceRaslR, // 9

    ReservedVclN10,
    ReservedVclR11,
    ReservedVclN12,
    ReservedVclR13,
    ReservedVclN14,
    ReservedVclR15,

    CodedSliceBlaWlp,   // 16
    CodedSliceBlaWRa,   // 17
    CodedSliceBlaNLP,   // 18
    CodedSliceIdrWRADL, // 19
    CodedSliceIdrNLP,   // 20
    CodedSliceCra,      // 21
    ReservedIrapVcl22,
    ReservedIrapVcl23,

    ReservedVcl24,
    ReservedVcl25,
    ReservedVcl26,
    ReservedVcl27,
    ReservedVcl28,
    ReservedVcl29,
    ReservedVcl30,
    ReservedVcl31,

    Vps,                 // 32
    Sps,                 // 33
    Pps,                 // 34
    AccessUnitDelimiter, // 35
    Eos,                 // 36
    Eob,                 // 37
    FilterData,          // 38
    PrefixSei,           // 39
    SuffixSei,           // 40
    ReservedNVcl41,
    ReservedNVcl42,
    ReservedNVcl43,
    ReservedNVcl44,
    ReservedNVcl45,
    ReservedNVcl46,
    ReservedNVcl47,
    Unspecified48,
    Unspecified49,
    Unspecified50,
    Unspecified51,
    Unspecified52,
    Unspecified53,
    Unspecified54,
    Unspecified55,
    Unspecified56,
    Unspecified57,
    Unspecified58,
    Unspecified59,
    Unspecified60,
    Unspecified61,
    Unspecified62,
    Unspecified63,
    Invalid,
}

impl NalUnitType {
    fn from_u8(n: u8) -> NalUnitType {
        match n {
            0 => NalUnitType::CodedSliceTrailN, // 0
            1 => NalUnitType::CodedSliceTrailR, // 1
            2 => NalUnitType::CodedSliceTsaN,   // 2
            3 => NalUnitType::CodedSliceTsaR,   // 3

            4 => NalUnitType::CodedSliceStsaN, // 4
            5 => NalUnitType::CodedSliceStsaR, // 5
            6 => NalUnitType::CodedSliceRadlN, // 6
            7 => NalUnitType::CodedSliceRadlR, // 7
            8 => NalUnitType::CodedSliceRaslN, // 8
            9 => NalUnitType::CodedSliceRaslR, // 9
            10 => NalUnitType::ReservedVclN10,
            11 => NalUnitType::ReservedVclR11,
            12 => NalUnitType::ReservedVclN12,
            13 => NalUnitType::ReservedVclR13,
            14 => NalUnitType::ReservedVclN14,
            15 => NalUnitType::ReservedVclR15,
            16 => NalUnitType::CodedSliceBlaWlp,   // 16
            17 => NalUnitType::CodedSliceBlaWRa,   // 17
            18 => NalUnitType::CodedSliceBlaNLP,   // 18
            19 => NalUnitType::CodedSliceIdrWRADL, // 19
            20 => NalUnitType::CodedSliceIdrNLP,   // 20
            21 => NalUnitType::CodedSliceCra,      // 21
            22 => NalUnitType::ReservedIrapVcl22,
            23 => NalUnitType::ReservedIrapVcl23,
            24 => NalUnitType::ReservedVcl24,
            25 => NalUnitType::ReservedVcl25,
            26 => NalUnitType::ReservedVcl26,
            27 => NalUnitType::ReservedVcl27,
            28 => NalUnitType::ReservedVcl28,
            29 => NalUnitType::ReservedVcl29,
            30 => NalUnitType::ReservedVcl30,
            31 => NalUnitType::ReservedVcl31,
            32 => NalUnitType::Vps,                 // 32
            33 => NalUnitType::Sps,                 // 33
            34 => NalUnitType::Pps,                 // 34
            35 => NalUnitType::AccessUnitDelimiter, // 35
            36 => NalUnitType::Eos,                 // 36
            37 => NalUnitType::Eob,                 // 37
            38 => NalUnitType::FilterData,          // 38
            39 => NalUnitType::PrefixSei,           // 39
            40 => NalUnitType::SuffixSei,           // 40
            41 => NalUnitType::ReservedNVcl41,
            42 => NalUnitType::ReservedNVcl42,
            43 => NalUnitType::ReservedNVcl43,
            44 => NalUnitType::ReservedNVcl44,
            45 => NalUnitType::ReservedNVcl45,
            46 => NalUnitType::ReservedNVcl46,
            47 => NalUnitType::ReservedNVcl47,
            48 => NalUnitType::Unspecified48,
            49 => NalUnitType::Unspecified49,
            50 => NalUnitType::Unspecified50,
            51 => NalUnitType::Unspecified51,
            52 => NalUnitType::Unspecified52,
            53 => NalUnitType::Unspecified53,
            54 => NalUnitType::Unspecified54,
            55 => NalUnitType::Unspecified55,
            56 => NalUnitType::Unspecified56,
            57 => NalUnitType::Unspecified57,
            58 => NalUnitType::Unspecified58,
            59 => NalUnitType::Unspecified59,
            60 => NalUnitType::Unspecified60,
            61 => NalUnitType::Unspecified61,
            62 => NalUnitType::Unspecified62,
            63 => NalUnitType::Unspecified63,
            _ => NalUnitType::Invalid,
        }
    }
}

#[derive(Debug)]
struct HevcNALArray {
    array_completeness: bool,
    nal_unit_type: NalUnitType,
    nal_list: Vec<Vec<u8>>,
}

pub struct ItemPropertyContainer {
    pub box_header: BoxHeader,
    pub properties: Vec<Box<ItemProperty>>,
}

impl ItemPropertyContainer {
    pub fn new(stream: &mut BitStream, box_header: BoxHeader) -> Result<Self> {
        if box_header.box_type != "ipco" {
            panic!("ipco");
        }
        let mut left = box_header.box_size - u64::from(box_header.header_size());
        let mut properties: Vec<Box<ItemProperty>> = Vec::new();
        while left > 0 {
            let sub_box_header = BoxHeader::new(stream)?;
            let property = if sub_box_header.box_type == "hvcC" {
                HevcConfigurationBox::new(stream, sub_box_header)?
            } else {
                unimplemented!("itemprop {}", sub_box_header.box_type)
            };
            left -= property.box_header.box_size;
            properties.push(Box::new(property));
        }
        Ok(Self {
            box_header,
            properties,
        })
    }
}

impl std::fmt::Debug for ItemPropertyContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ItemPropertyContainer {:?}", self.box_header)
    }
}