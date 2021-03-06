use std::any::Any;

use crate::bit::Byte4;

pub mod ftyp;
pub mod header;
pub mod meta;
pub mod moov;

pub trait BBox {
    fn box_type(&self) -> &Byte4;
    fn as_any(&self) -> &dyn Any;
}
