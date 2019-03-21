use std::collections::HashMap;
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
            } else if header.box_type == "mdat" {
                println!(">>SKIPPING {:?}", header);
                stream.skip_bytes(header.body_size() as usize)?;
            } else if header.box_type == "moov" {
                if movie_box.is_some() {
                    return Err(HeifError::InvalidFormat);
                }
                let mut ex = stream.extract_from(&header)?;
                movie_box = Some(MovieBox::new(&mut ex, header)?);
            } else {
                panic!("unknown type {}", header.box_type)
            }
        }
        if ftyp.is_none() || (metabox_map.get(&0).is_none() && movie_box.is_none()) {
            return Err(HeifError::InvalidFormat);
        }
        let ftyp = ftyp.unwrap();
        Ok(HeifReader { ftyp, metabox_map })
    }
}
