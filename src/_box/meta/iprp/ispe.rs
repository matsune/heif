use crate::_box::meta::iprp::ItemProperty;
use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
pub struct ImageSpatialExtentsProperty {
    pub full_box_header: FullBoxHeader,
    pub image_width: u32,
    pub image_height: u32,
}

impl ImageSpatialExtentsProperty {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let image_width = stream.read_4bytes()?.to_u32();
        let image_height = stream.read_4bytes()?.to_u32();
        Ok(Self {
            full_box_header,
            image_width,
            image_height,
        })
    }
}

impl ItemProperty for ImageSpatialExtentsProperty {}
