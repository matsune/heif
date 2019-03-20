// mod _box;
mod bit;

use std::fs::File;

use bit::BitStream;
// use _box::BoxHeader;
// use _box::ftyp::FileTypeBox;

pub type Result<T> = std::result::Result<T, HeifError>;

#[derive(Debug)]
pub enum HeifError {
    FileOpen,
    FileRead,
    EOF,
}

impl HeifError {
    fn __description(&self) -> &str {
        match *self {
            HeifError::FileOpen => "FileOpen",
            HeifError::FileRead => "FileRead",
            HeifError::EOF => "EOF",
            _ => "Unknown HeifError",
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

    // while !stream.is_eof() {
    //     let header = BoxHeader::new(&mut stream)?;
    //     if header.box_type == "ftyp" {
    //         if ftyp_found {
    //             // FIXME
    //             panic!("already has ftyp");
    //         }
    //         ftyp_found = true;
    //         let ft_box = FileTypeBox::new(&mut stream, header)?;
    //         println!("{:?}", ft_box);
    //         } else if header.box_type == "meta" {
    //             if meta_found {
    //                 // FIXME
    //                 panic!("already has meta");
    //                 // return Err(HeifError::FileRead);
    //             }
    //             meta_found = true;
    //             let m_box = MetaBox::new(&mut stream, header)?;
    //             println!("meta {:?}", m_box);
    // } else {
    //     // TODO: skip_box();
    //     panic!("unimplemented box type {}", header.box_type)
    // }
    // }
    Ok(())
}
