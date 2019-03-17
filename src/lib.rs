mod bit;
mod bbox;

use std::fs::File;
use std::io::{Result};

use crate::bit::{BitStream};
use crate::bbox::{BoxHeader};
use crate::bbox::ftyp::FileTypeBox;
use crate::bbox::meta::*;

pub fn load(file_path: &str) -> Result<()> {
    let file = File::open(file_path)?;
    let mut stream = BitStream::new(file)?;
    let mut ftyp_found = false;
    let mut meta_found = false;

    while !stream.is_eof() {
        let header = BoxHeader::new(&mut stream)?;
        if header.box_type == "ftyp" {
            if ftyp_found {
              // FIXME
              panic!("already has ftyp");
            }
            ftyp_found = true;
            let ft_box = FileTypeBox::new(&mut stream, header)?;
            println!("{:?}", ft_box);
        } else if header.box_type == "meta" {
            if meta_found {
              // FIXME
              panic!("already has meta");
                // return Err(HeifError::FileRead);
            }
            meta_found = true;
            let m_box = MetaBox::new(&mut stream, header)?;
            println!("meta {:?}", m_box);
        } else {
            // TODO: skip_box();
            panic!("unimplemented box type {}", header.box_type)
        }
    }
    Ok(())
}