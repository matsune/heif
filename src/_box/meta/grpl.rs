use crate::_box::{BoxHeader, FullBoxHeader};
use crate::bit::Stream;
use crate::Result;

#[derive(Debug)]
pub struct GroupListBox {
    pub box_header: BoxHeader,
    pub entity_to_group_box_vector: Vec<EntityToGroupBox>,
}
impl GroupListBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut entity_to_group_box_vector = Vec::new();
        while !stream.is_eof() {
            let mut ex = stream.extract_from(&box_header)?;
            let child_box_header = BoxHeader::new(&mut ex)?;
            entity_to_group_box_vector.push(EntityToGroupBox::new(&mut ex, child_box_header)?);
        }
        Ok(Self {
            box_header,
            entity_to_group_box_vector,
        })
    }
}
#[derive(Debug)]
pub struct EntityToGroupBox {
    full_box_header: FullBoxHeader,
    group_id: u32,
    entity_ids: Vec<u32>,
}

impl EntityToGroupBox {
    pub fn new<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::new(stream, box_header)?;
        let group_id = stream.read_4bytes()?.to_u32();
        let entity_count = stream.read_4bytes()?.to_u32();
        let mut entity_ids = Vec::new();
        for _ in 0..entity_count {
            entity_ids.push(stream.read_4bytes()?.to_u32());
        }
        Ok(Self {
            full_box_header,
            group_id,
            entity_ids,
        })
    }
}
