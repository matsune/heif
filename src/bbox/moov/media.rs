use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::meta::dinf::DataEntryBox;

#[derive(Debug)]
pub struct MediaBox {
    box_header: BoxHeader,
    media_header_box: MediaHeaderBox,
    handler_box: HandlerBox,
    media_information_box: MediaInformationBox,
}

#[derive(Debug)]
pub struct MediaHeaderBox {
    full_box_header: FullBoxHeader,
    creation_time: u64,
    modification_time: u64,
    time_scale: u32,
    duration: u64,
    language: u16,
}

#[derive(Debug)]
pub struct HandlerBox {
    full_box_header: FullBoxHeader,
    handler_type: String,
    name: String,
}

#[derive(Debug)]
enum MediaType {
    Null,
    Video,
    Sound,
}

#[derive(Debug)]
pub struct MediaInformationBox {
    box_header: BoxHeader,
    media_type: MediaType,
    video_media_header_box: VideoMediaHeaderBox,
    sound_media_header_box: SoundMediaHeaderBox,
    null_media_header_box: NullMediaHeaderBox,
    data_information_box: DataInformationBox,
    sample_table_box: SampleTableBox,
}

#[derive(Debug)]
struct VideoMediaHeaderBox {
    full_box_header: FullBoxHeader,
}

#[derive(Debug)]
struct SoundMediaHeaderBox {
    full_box_header: FullBoxHeader,
    balance: u16,
}

#[derive(Debug)]
struct NullMediaHeaderBox {
    full_box_header: FullBoxHeader,
}

#[derive(Debug)]
struct DataInformationBox {
    box_header: BoxHeader,
    data_reference_box: DataReferenceBox,
}

struct DataReferenceBox {
    full_box_header: FullBoxHeader,
    data_entries: Vec<DataEntryBox>,
}

impl std::fmt::Debug for DataReferenceBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DataReferenceBox {:?}", self.full_box_header)
    }
}

#[derive(Debug)]
struct SampleTableBox {
    box_header: BoxHeader,
    sample_description_box: SampleDescriptionBox,
}

#[derive(Debug)]
struct SampleDescriptionBox {
    full_box_header: FullBoxHeader,
    index: Vec<SampleEntryBox>,
}

#[derive(Debug)]
struct SampleEntryBox {
    box_header: BoxHeader,
    data_reference_index: u16,
}
