use crate::Result;
use crate::_box::moov::media::MediaBox;
use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;

#[derive(Debug)]
pub struct TrackBox {
    box_header: BoxHeader,
    track_header_box: TrackHeaderBox,
    media_box: MediaBox,
    track_reference_box: TrackReferenceBox,
    has_track_references: bool,
    edit_box: EditBox,
}

#[derive(Debug)]
pub struct TrackHeaderBox {
    full_box_header: FullBoxHeader,
    creation_time: u64,
    modification_time: u64,
    track_id: u32,
    duration: u64,
    width: u32,
    height: u32,
    alternate_group: u16,
    volume: u16,
    matrix: Vec<i32>,
}

#[derive(Debug)]
struct TrackReferenceTypeBox {
    box_header: BoxHeader,
    track_id: Vec<u32>,
}

#[derive(Debug)]
pub struct TrackReferenceBox {
    box_header: BoxHeader,
    track_ref_type_boxes: Vec<TrackReferenceTypeBox>,
}

#[derive(Debug)]
struct EditBox {
    full_box_header: FullBoxHeader,
    entry_version_0: Vec<EntryVersion0>,
    entry_version_1: Vec<EntryVersion1>,
}

#[derive(Debug)]
struct EntryVersion0 {
    segment_duration: u32,
    media_time: i32,
    media_rate_integer: u16,
    media_rate_fraction: u16,
}

#[derive(Debug)]
struct EntryVersion1 {
    segment_duration: u64,
    media_time: i64,
    media_rate_integer: u16,
    media_rate_fraction: u16,
}
