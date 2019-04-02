use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct ImageSpatialExtentsProperty {
    full_box_header: FullBoxHeader,
    width: u32,
    height: u32,
}

impl Default for ImageSpatialExtentsProperty {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl BBox for ImageSpatialExtentsProperty {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ImageSpatialExtentsProperty {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            full_box_header: FullBoxHeader::new("ispe".parse().unwrap(), 0, 0),
            width,
            height,
        }
    }

    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
        let width = stream.read_4bytes()?.to_u32();
        let height = stream.read_4bytes()?.to_u32();
        Ok(Self {
            full_box_header,
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn set_width(&mut self, w: u32) {
        self.width = w;
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_height(&mut self, h: u32) {
        self.height = h;
    }
}
