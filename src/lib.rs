mod _box;
mod bit;
pub mod error;
pub mod reader;

pub type Result<T> = std::result::Result<T, error::HeifError>;
