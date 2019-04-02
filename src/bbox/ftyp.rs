use crate::bbox::header::BoxHeader;
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

#[derive(Debug)]
pub struct FileTypeBox {
    box_header: BoxHeader,
    major_brand: Byte4,
    minor_version: u32,
    compatible_brands: Vec<Byte4>,
}

impl std::default::Default for FileTypeBox {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new("ftyp".parse().unwrap()),
            major_brand: Byte4::default(),
            minor_version: 0,
            compatible_brands: Vec::new(),
        }
    }
}

impl BBox for FileTypeBox {
    fn box_type(&self) -> &Byte4 {
        self.box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FileTypeBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let box_header = box_header;
        let major_brand = stream.read_4bytes()?;
        let minor_version = stream.read_4bytes()?.to_u32();
        let mut compatible_brands = Vec::new();
        while !stream.is_eof() {
            compatible_brands.push(stream.read_4bytes()?);
        }
        Ok(Self {
            box_header,
            major_brand,
            minor_version,
            compatible_brands,
        })
    }

    pub fn major_brand(&self) -> &Byte4 {
        &self.major_brand
    }

    pub fn set_major_brand(&mut self, brand: Byte4) {
        self.major_brand = brand;
    }

    pub fn minor_version(&self) -> u32 {
        self.minor_version
    }

    pub fn set_minor_version(&mut self, ver: u32) {
        self.minor_version = ver;
    }

    pub fn add_compatible_brand(&mut self, brand: Byte4) {
        if !self.is_compatible_brand(&brand) {
            self.compatible_brands.push(brand);
        }
    }

    pub fn compatible_brands(&self) -> &Vec<Byte4> {
        &self.compatible_brands
    }

    pub fn is_compatible_brand(&self, brand: &Byte4) -> bool {
        self.compatible_brands.contains(brand)
    }
}
