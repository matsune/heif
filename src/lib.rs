mod _box;
mod bit;

use std::fs::File;

use _box::ftyp::FileTypeBox;
use _box::meta::MetaBox;
use _box::{BoxHeader, Header};
use bit::{BitStream, Stream};

pub type Result<T> = std::result::Result<T, HeifError>;

#[derive(Debug)]
pub enum HeifError {
    FileOpen,
    FileRead,
    InvalidFormat,
    EOF,
}

impl HeifError {
    fn __description(&self) -> &str {
        match *self {
            HeifError::FileOpen => "FileOpen",
            HeifError::FileRead => "FileRead",
            HeifError::InvalidFormat => "InvalidFormat",
            HeifError::EOF => "EOF",
        }
    }
}

impl std::fmt::Display for HeifError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.__description())
    }
}

impl std::error::Error for HeifError {
    fn description(&self) -> &str {
        self.__description()
    }
}

pub fn load(file_path: &str) -> Result<()> {
    let mut file = File::open(file_path).map_err(|_| HeifError::FileOpen)?;
    let mut stream = BitStream::from(&mut file)?;
    let mut ftyp_found = false;
    let mut meta_found = false;

    while !stream.is_eof() {
        let header = BoxHeader::new(&mut stream)?;
        if header.box_type == "ftyp" {
            if ftyp_found {
                return Err(HeifError::InvalidFormat);
            }
            ftyp_found = true;
            let mut ex = stream.extract_from(&header)?;
            let ft_box = FileTypeBox::new(&mut ex, header)?;
            println!("{:?}", ft_box);
        } else if header.box_type == "meta" {
            if meta_found {
                return Err(HeifError::InvalidFormat);
            }
            meta_found = true;
            let mut ex = stream.extract_from(&header)?;
            let m_box = MetaBox::new(&mut ex, header)?;
            println!("{:?}", m_box);
        } else if header.box_type == "mdat" {
            println!(">>SKIPPING {:?}", header);
            stream.skip_bytes(header.get_box_size() as usize)?;
        } else {
            panic!("unknown type {}", header.box_type)
        }
    }
    Ok(())
}
