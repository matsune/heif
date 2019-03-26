pub mod ftyp;
pub mod header;
pub mod meta;
pub mod moov;

pub trait BBox {
    type HeaderType;
    fn header(&self) -> &Self::HeaderType;
}
