use crate::bbox::header::{BoxHeader, FullBoxHeader};
use crate::bbox::BBox;
use crate::bit::{Byte4, Stream};
use crate::{HeifError, Result};

#[derive(Debug)]
pub struct ItemReferenceBox {
    full_box_header: FullBoxHeader,
    reference_list: Vec<SingleItemTypeReferenceBox>,
}

impl Default for ItemReferenceBox {
    fn default() -> Self {
        Self {
            full_box_header: FullBoxHeader::new("iref".parse().unwrap(), 0, 0),
            reference_list: Vec::new(),
        }
    }
}

impl BBox for ItemReferenceBox {
    fn box_type(&self) -> &Byte4 {
        self.full_box_header.box_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemReferenceBox {
    fn full_box_header(&self) -> &FullBoxHeader {
        &self.full_box_header
    }

    pub fn from_stream_header<T: Stream>(stream: &mut T, box_header: BoxHeader) -> Result<Self> {
        let full_box_header = FullBoxHeader::from_stream_header(stream, box_header)?;
        let is_large = full_box_header.version() > 0;
        let mut reference_list = Vec::new();
        while !stream.is_eof() {
            let box_header = BoxHeader::from_stream(stream)?;
            reference_list.push(SingleItemTypeReferenceBox::from_stream_is_large(
                stream, box_header, is_large,
            )?);
        }
        Ok(Self {
            full_box_header,
            reference_list,
        })
    }

    pub fn add_item_ref(&mut self, ref_box: SingleItemTypeReferenceBox) {
        self.reference_list.push(ref_box);
    }

    pub fn references_of_type(&self, box_type: Byte4) -> Vec<&SingleItemTypeReferenceBox> {
        self.reference_list
            .iter()
            .filter(|r| *r.box_header().box_type() == box_type)
            .collect()
    }

    pub fn add(&mut self, box_type: Byte4, from_id: u32, to_id: u32) -> Result<()> {
        let is_large = self.full_box_header.version() != 0;
        if (from_id > std::u16::MAX.into() || to_id > std::u16::MAX.into()) && !is_large {
            return Err(HeifError::InvalidItemID);
        }
        if let Some(item_ref) = self
            .reference_list
            .iter_mut()
            .find(|i| *i.box_header().box_type() == box_type && i.get_from_item_id() == from_id)
        {
            item_ref.add_to_item_id(to_id);
        } else {
            let mut item_ref = SingleItemTypeReferenceBox::new(is_large);
            item_ref.set_reference_type(box_type);
            item_ref.set_from_item_id(from_id);
            item_ref.add_to_item_id(to_id);
            self.reference_list.push(item_ref);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SingleItemTypeReferenceBox {
    box_header: BoxHeader,
    from_item_id: u32,
    to_item_ids: Vec<u32>,
    is_large: bool,
}

impl SingleItemTypeReferenceBox {
    pub fn new(is_large: bool) -> Self {
        Self {
            box_header: BoxHeader::new(Byte4::default()),
            from_item_id: 0,
            to_item_ids: Vec::new(),
            is_large,
        }
    }

    pub fn from_stream_is_large<T: Stream>(
        stream: &mut T,
        box_header: BoxHeader,
        is_large: bool,
    ) -> Result<Self> {
        let from_item_id = if is_large {
            stream.read_4bytes()?.to_u32()
        } else {
            stream.read_2bytes()?.to_u32()
        };
        let ref_count = stream.read_2bytes()?.to_u16();
        let mut to_item_ids = Vec::new();
        for _ in 0..ref_count {
            to_item_ids.push(if is_large {
                stream.read_4bytes()?.to_u32()
            } else {
                stream.read_2bytes()?.to_u32()
            })
        }
        Ok(Self {
            box_header,
            from_item_id,
            to_item_ids,
            is_large,
        })
    }

    fn box_header(&self) -> &BoxHeader {
        &self.box_header
    }

    pub fn set_reference_type(&mut self, r_type: Byte4) {
        self.box_header.set_box_type(r_type);
    }

    pub fn get_from_item_id(&self) -> u32 {
        self.from_item_id
    }

    pub fn set_from_item_id(&mut self, id: u32) {
        self.from_item_id = id;
    }

    pub fn add_to_item_id(&mut self, id: u32) {
        self.to_item_ids.push(id);
    }

    pub fn clear_to_item_id(&mut self) {
        self.to_item_ids.clear();
    }

    pub fn to_item_ids(&self) -> &Vec<u32> {
        &self.to_item_ids
    }
}
