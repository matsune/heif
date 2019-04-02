use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::Result;

#[derive(Debug)]
pub struct GroupListBox {
    box_header: BoxHeader,
    entity_to_group_box_vector: Vec<EntityToGroupBox>,
}

impl Default for GroupListBox {
    fn default() -> Self {
        Self {
            box_header: BoxHeader::new("grpl".parse().unwrap()),
            entity_to_group_box_vector: Vec::new(),
        }
    }
}

impl BBox for GroupListBox {
    fn box_type(&self) -> &Byte4 {
        self.box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl GroupListBox {
    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let mut entity_to_group_box_vector = Vec::new();
        while !stream.is_eof() {
            let mut ex = stream.extract_from(&box_header)?;
            let child_box_header = BoxHeader::from_stream(&mut ex)?;
            entity_to_group_box_vector.push(EntityToGroupBox::new(&mut ex, child_box_header)?);
        }
        Ok(Self {
            box_header,
            entity_to_group_box_vector,
        })
    }

    pub fn box_header(&self) -> &BoxHeader {
        &self.box_header
    }

    pub fn entity_to_group_box_vector(&self) -> &Vec<EntityToGroupBox> {
        &self.entity_to_group_box_vector
    }

    pub fn add_entity_to_group_box(&mut self, entity: EntityToGroupBox) {
        self.entity_to_group_box_vector.push(entity)
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
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
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

    pub fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn group_id(&self) -> u32 {
        self.group_id
    }

    pub fn set_group_id(&mut self, id: u32) {
        self.group_id = id;
    }

    pub fn entity_ids(&self) -> &Vec<u32> {
        &self.entity_ids
    }

    pub fn set_entity_ids(&mut self, ids: Vec<u32>) {
        self.entity_ids = ids;
    }
}
