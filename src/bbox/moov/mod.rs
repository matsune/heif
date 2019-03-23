pub mod media;
pub mod track;

use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;
use track::TrackBox;

#[derive(Debug)]
pub struct MovieBox {
    movie_header_box: MovieHeaderBox,
    tracks: Vec<TrackBox>,
    is_ozo_preview_file: bool,
}

impl MovieBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        unimplemented!("MovieBox")
    }
}

#[derive(Debug)]
pub struct MovieHeaderBox {
    full_box_header: FullBoxHeader,
    creation_time: u64,
    modification_time: u64,
    time_scale: u32,
    duration: u32,
    matrix: Vec<i32>,
    next_track_id: u32,
}
