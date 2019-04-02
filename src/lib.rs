mod bbox;
mod bit;
mod data;
mod internal;
pub mod reader;

pub type Result<T> = std::result::Result<T, HeifError>;

#[derive(Debug)]
pub enum HeifError {
    FileOpen,
    FileRead,
    FileHeader,
    InvalidItemID,
    Uninitialized,
    EOF,
    Unknown(&'static str),
}

impl HeifError {
    fn __description(&self) -> &str {
        match *self {
            HeifError::FileOpen => "FileOpen",
            HeifError::FileRead => "FileRead",
            HeifError::FileHeader => "FileHeader",
            HeifError::InvalidItemID => "InvalidItemID",
            HeifError::Uninitialized => "Uninitialized",
            HeifError::EOF => "EOF",
            HeifError::Unknown(s) => s,
        }
    }
}

impl std::fmt::Display for HeifError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.__description())
    }
}

impl std::error::Error for HeifError {
    fn description(&self) -> &str {
        self.__description()
    }
}
