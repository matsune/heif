use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::{HeifError, Result};

use std::str::FromStr;

#[derive(Debug)]
pub struct HevcConfigurationBox {
    box_header: BoxHeader,
    hevc_config: HevcDecoderConfigurationRecord,
}

impl Default for HevcConfigurationBox {
    fn default() -> Self {
        Self::new(HevcDecoderConfigurationRecord::default())
    }
}

impl BBox for HevcConfigurationBox {
    fn box_type(&self) -> &Byte4 {
        self.box_header.box_type()
    }
}

impl HevcConfigurationBox {
    pub fn new(hevc_config: HevcDecoderConfigurationRecord) -> Self {
        Self {
            box_header: BoxHeader::new(Byte4::from_str("hvcC").unwrap()),
            hevc_config,
        }
    }

    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        Ok(Self {
            box_header,
            hevc_config: HevcDecoderConfigurationRecord::from_stream(stream)?,
        })
    }
}

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
    nal_array: Vec<NalArray>,
}

impl Default for HevcDecoderConfigurationRecord {
    fn default() -> Self {
        Self {
            configuration_version: 1,
            general_profile_space: 0,
            general_tier_flag: 0,
            general_profile_idc: 0,
            general_profile_compatibility_flags: 0,
            general_constraint_indicator_flags: [0; 6],
            general_level_idc: 0,
            min_spatial_segmentation_idc: 0,
            parallelism_type: 0,
            chroma_format: 0,
            pic_width_in_luma_samples: 0,
            pic_height_in_luma_samples: 0,
            conf_win_left_offset: 0,
            conf_win_right_offset: 0,
            conf_win_top_offset: 0,
            conf_win_bottom_offset: 0,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            avg_frame_rate: 0,
            constant_frame_rate: 0,
            num_temporal_layers: 0,
            temporal_id_nested: 0,
            length_size_minus1: 0,
            nal_array: Vec::new(),
        }
    }
}

impl HevcDecoderConfigurationRecord {
    fn from_stream<T: Stream>(stream: &mut T) -> Result<Self> {
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
                res.add_nal_unit(nal_data, nal_unit_type, array_completeness);
            }
        }
        Ok(res)
    }

    fn add_nal_unit(
        &mut self,
        nal_unit: Vec<u8>,
        nal_unit_type: NalUnitType,
        array_completeness: bool,
    ) {
        let start_code_len = find_start_code_len(&nal_unit);
        let v = nal_unit[start_code_len..].to_vec();
        match self
            .nal_array
            .iter_mut()
            .find(|unit| unit.nal_unit_type == nal_unit_type)
        {
            Some(n) => {
                n.nal_list.push(v);
            }
            None => {
                let tmp = NalArray {
                    array_completeness,
                    nal_unit_type,
                    nal_list: vec![v],
                };
                self.nal_array.push(tmp);
            }
        };
    }
}

fn find_start_code_len(data: &[u8]) -> usize {
    let mut i = 0;
    let size = data.len();
    while (i + 1) < size && data[i] == 0 {
        i += 1;
    }
    if i > 1 && data[i] == 1 {
        i + 1
    } else {
        0
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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
struct NalArray {
    array_completeness: bool,
    nal_unit_type: NalUnitType,
    nal_list: Vec<Vec<u8>>,
}
