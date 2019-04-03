use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use crate::bbox::header::Header;
use crate::{HeifError, Result};

#[derive(Debug)]
pub struct ParseBytesError;

#[derive(Default, Debug, PartialEq)]
pub struct Byte2(pub u8, pub u8);

impl Byte2 {
    pub fn to_u16(&self) -> u16 {
        (u16::from(self.0) << 8) + u16::from(self.1)
    }

    pub fn to_u32(&self) -> u32 {
        u32::from(self.to_u16())
    }

    pub fn to_u64(&self) -> u64 {
        u64::from(self.to_u16())
    }

    pub fn to_usize(&self) -> usize {
        usize::from(self.to_u16())
    }
}

impl FromStr for Byte2 {
    type Err = ParseBytesError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return Err(Self::Err {});
        }
        Ok(Self(bytes[0], bytes[1]))
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Byte4(pub u8, pub u8, pub u8, pub u8);

impl Byte4 {
    pub fn to_u32(&self) -> u32 {
        (u32::from(self.0) << 24)
            + (u32::from(self.1) << 16)
            + (u32::from(self.2) << 8)
            + u32::from(self.3)
    }

    pub fn to_u64(&self) -> u64 {
        u64::from(self.to_u32())
    }

    pub fn to_usize(&self) -> usize {
        self.to_u32() as usize
    }
}

impl std::fmt::Debug for Byte4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Byte4(\"{}\")", self.to_string())
    }
}

impl FromStr for Byte4 {
    type Err = ParseBytesError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            return Err(Self::Err {});
        }
        Ok(Self(bytes[0], bytes[1], bytes[2], bytes[3]))
    }
}

impl ToString for Byte4 {
    fn to_string(&self) -> String {
        format!(
            "{}{}{}{}",
            self.0 as char, self.1 as char, self.2 as char, self.3 as char
        )
    }
}

impl std::cmp::PartialEq<&str> for Byte4 {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().as_str() == *other
    }
}

impl std::cmp::PartialEq<str> for Byte4 {
    fn eq(&self, other: &str) -> bool {
        self.to_string().as_str() == other
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Byte8(
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
);

impl Byte8 {
    pub fn to_u64(&self) -> u64 {
        (u64::from(self.0) << 56)
            + (u64::from(self.1) << 48)
            + (u64::from(self.2) << 40)
            + (u64::from(self.3) << 32)
            + (u64::from(self.4) << 24)
            + (u64::from(self.5) << 16)
            + (u64::from(self.6) << 8)
            + u64::from(self.7)
    }
}

pub trait Stream {
    fn len(&self) -> usize;
    fn get_byte_offset(&self) -> usize;
    fn set_byte_offset(&mut self, n: usize);
    fn get_bit_offset(&self) -> u8;
    fn set_bit_offset(&mut self, n: u8);
    fn byte_at(&self, n: usize) -> u8;
    fn read_bytes(&mut self, size: usize) -> Result<&[u8]>;

    fn num_bytes_left(&self) -> usize {
        self.len() - self.get_byte_offset()
    }

    fn has_bytes(&self, n: usize) -> bool {
        self.num_bytes_left() >= n
    }

    fn read_byte(&mut self) -> Result<u8> {
        if !self.has_bytes(1) {
            return Err(HeifError::EOF);
        }
        let byte = self.byte_at(self.get_byte_offset());
        self.set_byte_offset(self.get_byte_offset() + 1);
        Ok(byte)
    }

    fn read_2bytes(&mut self) -> Result<Byte2> {
        if !self.has_bytes(2) {
            return Err(HeifError::EOF);
        }
        let byte2 = Byte2(
            self.byte_at(self.get_byte_offset()),
            self.byte_at(self.get_byte_offset() + 1),
        );
        self.set_byte_offset(self.get_byte_offset() + 2);
        Ok(byte2)
    }

    fn read_4bytes(&mut self) -> Result<Byte4> {
        if !self.has_bytes(4) {
            return Err(HeifError::EOF);
        }
        let byte4 = Byte4(
            self.byte_at(self.get_byte_offset()),
            self.byte_at(self.get_byte_offset() + 1),
            self.byte_at(self.get_byte_offset() + 2),
            self.byte_at(self.get_byte_offset() + 3),
        );
        self.set_byte_offset(self.get_byte_offset() + 4);
        Ok(byte4)
    }

    fn read_8bytes(&mut self) -> Result<Byte8> {
        if !self.has_bytes(8) {
            return Err(HeifError::EOF);
        }
        let byte8 = Byte8(
            self.byte_at(self.get_byte_offset()),
            self.byte_at(self.get_byte_offset() + 1),
            self.byte_at(self.get_byte_offset() + 2),
            self.byte_at(self.get_byte_offset() + 3),
            self.byte_at(self.get_byte_offset() + 4),
            self.byte_at(self.get_byte_offset() + 5),
            self.byte_at(self.get_byte_offset() + 6),
            self.byte_at(self.get_byte_offset() + 7),
        );
        self.set_byte_offset(self.get_byte_offset() + 8);
        Ok(byte8)
    }

    fn skip_bytes(&mut self, n: usize) -> Result<usize> {
        let left = self.num_bytes_left();
        if n > left {
            return Err(HeifError::EOF);
        }
        self.set_byte_offset(self.get_byte_offset() + n);
        Ok(self.get_byte_offset())
    }

    fn is_eof(&self) -> bool {
        self.get_byte_offset() >= self.len()
    }

    fn current_byte(&mut self) -> Result<u8> {
        if self.is_eof() {
            return Err(HeifError::EOF);
        }
        Ok(self.byte_at(self.get_byte_offset()))
    }

    fn read_bits(&mut self, n: usize) -> Result<usize> {
        if n == 0 {
            return Ok(0);
        }
        let mut return_bits;
        let num_bits_left_in_byte = usize::from(8 - self.get_bit_offset());
        if num_bits_left_in_byte >= n {
            return_bits =
                usize::from(self.current_byte()? >> (num_bits_left_in_byte - n)) & ((1 << n) - 1);
            self.set_bit_offset(self.get_bit_offset() + n as u8);
        } else {
            let mut num_bits_togo = n - num_bits_left_in_byte;
            return_bits = usize::from(self.current_byte()?) & ((1 << num_bits_left_in_byte) - 1);
            self.skip_bytes(1)?;
            self.set_bit_offset(0);
            while num_bits_togo > 0 {
                if num_bits_togo > 8 {
                    return_bits = (return_bits << 8) | usize::from(self.current_byte()?);
                    self.skip_bytes(1)?;
                    num_bits_togo -= 8;
                } else {
                    return_bits = (return_bits << num_bits_togo)
                        | (usize::from(self.current_byte()? >> (8 - num_bits_togo))
                            & ((1 << num_bits_togo) - 1));
                    self.set_bit_offset(self.get_bit_offset() + num_bits_togo as u8);
                    num_bits_togo = 0;
                }
            }
        }
        if self.get_bit_offset() == 8 {
            self.skip_bytes(1)?;
            self.set_bit_offset(0);
        }
        Ok(return_bits)
    }

    fn read_zero_term_string(&mut self) -> String {
        let mut string = String::new();
        while !self.is_eof() {
            let ch = self.read_byte().unwrap();
            if ch == 0 {
                break;
            } else {
                string.push(char::from(ch));
            }
        }
        string
    }

    fn extract(&mut self, size: usize) -> Result<Extract> {
        Ok(Extract::new(self.read_bytes(size)?))
    }

    fn extract_from<H: Header>(&mut self, header: &H) -> Result<Extract> {
        self.extract(header.body_size() as usize)
    }
}

#[derive(Debug)]
pub struct Extract<'a> {
    inner: &'a [u8],
    bit_offset: u8,
    byte_offset: usize,
}

impl<'a> Extract<'a> {
    pub fn new(inner: &'a [u8]) -> Self {
        Self {
            inner,
            bit_offset: 0,
            byte_offset: 0,
        }
    }
}

impl<'a> Stream for Extract<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }

    fn get_byte_offset(&self) -> usize {
        self.byte_offset
    }

    fn set_byte_offset(&mut self, n: usize) {
        self.byte_offset = n
    }

    fn get_bit_offset(&self) -> u8 {
        self.bit_offset
    }

    fn set_bit_offset(&mut self, n: u8) {
        self.bit_offset = n;
    }

    fn byte_at(&self, n: usize) -> u8 {
        self.inner[n]
    }

    fn read_bytes(&mut self, size: usize) -> Result<&[u8]> {
        if !self.has_bytes(size) {
            return Err(HeifError::EOF);
        }
        let slice = &self.inner[self.byte_offset..(self.byte_offset + size)];
        self.byte_offset += size;
        Ok(slice)
    }
}

#[derive(Debug, Default)]
pub struct BitStream {
    inner: Vec<u8>,
    bit_offset: u8,
    byte_offset: usize,
}

impl BitStream {
    pub fn new(inner: Vec<u8>) -> Self {
        Self {
            inner,
            bit_offset: 0,
            byte_offset: 0,
        }
    }

    pub fn from(file: &mut File) -> Result<Self> {
        let mut inner = Vec::new();
        file.read_to_end(&mut inner)
            .map_err(|_| HeifError::FileRead)?;
        Ok(BitStream::new(inner))
    }
}

impl Stream for BitStream {
    fn len(&self) -> usize {
        self.inner.len()
    }

    fn get_byte_offset(&self) -> usize {
        self.byte_offset
    }

    fn set_byte_offset(&mut self, n: usize) {
        self.byte_offset = n
    }

    fn get_bit_offset(&self) -> u8 {
        self.bit_offset
    }

    fn set_bit_offset(&mut self, n: u8) {
        self.bit_offset = n;
    }

    fn byte_at(&self, n: usize) -> u8 {
        self.inner[n]
    }

    fn read_bytes(&mut self, size: usize) -> Result<&[u8]> {
        if !self.has_bytes(size) {
            return Err(HeifError::EOF);
        }
        let slice = &self.inner[self.byte_offset..(self.byte_offset + size)];
        self.byte_offset += size;
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod byte2 {
        use super::*;

        #[test]
        fn test_from_str() {
            assert_eq!(Byte2::from_str("ab").unwrap(), Byte2(0x61, 0x62));
            assert!(Byte2::from_str("a").is_err());
            assert!(Byte2::from_str("abc").is_err());
        }
    }

    mod byte4 {
        use super::*;

        #[test]
        fn test_from_str() {
            assert_eq!("abcd".parse().unwrap(), Byte4(0x61, 0x62, 0x63, 0x64));
            assert!(Byte4::from_str("a").is_err());
            assert!(Byte4::from_str("abcde").is_err());
        }

        #[test]
        fn test_to_string() {
            assert_eq!(Byte4(0x66, 0x74, 0x79, 0x70).to_string(), "ftyp");
        }

        #[test]
        fn test_eq_string() {
            assert!(Byte4(0x66, 0x74, 0x79, 0x70) == "ftyp");
        }
    }

    mod stream {
        use super::*;

        #[test]
        fn test_num_bytes_left() {
            let mut stream = BitStream::new(vec![10, 11]);
            assert_eq!(stream.num_bytes_left(), 2);
            stream.byte_offset += 1;
            assert_eq!(stream.num_bytes_left(), 1);
            stream.byte_offset += 1;
            assert_eq!(stream.num_bytes_left(), 0);
        }

        #[test]
        fn test_has_bytes() {
            let mut stream = BitStream::new(vec![10, 11]);
            assert!(stream.has_bytes(2));
            stream.byte_offset += 1;
            assert!(stream.has_bytes(1));
            stream.byte_offset += 1;
            assert!(!stream.has_bytes(1));
        }

        #[test]
        fn test_read_byte() {
            let mut stream = BitStream::new(vec![10, 11]);
            assert_eq!(stream.byte_offset, 0);
            assert_eq!(stream.read_byte().unwrap(), 10);
            assert_eq!(stream.byte_offset, 1);
            assert_eq!(stream.read_byte().unwrap(), 11);
            assert_eq!(stream.byte_offset, 2);
            assert!(stream.read_byte().is_err());
        }

        #[test]
        fn test_read_2bytes() {
            let mut stream = BitStream::new(vec![10, 11, 12]);
            assert_eq!(stream.byte_offset, 0);
            assert_eq!(stream.read_2bytes().unwrap(), Byte2(10, 11));
            assert_eq!(stream.byte_offset, 2);
            assert!(stream.read_2bytes().is_err());
        }

        #[test]
        fn test_read_4bytes() {
            let mut stream = BitStream::new(vec![10, 11, 12, 13]);
            assert_eq!(stream.byte_offset, 0);
            assert_eq!(stream.read_4bytes().unwrap(), Byte4(10, 11, 12, 13));
            assert_eq!(stream.byte_offset, 4);
            assert!(stream.read_4bytes().is_err());
        }

        #[test]
        fn test_read_8bytes() {
            let mut stream = BitStream::new(vec![10, 11, 12, 13, 14, 15, 16, 17]);
            assert_eq!(stream.byte_offset, 0);
            assert_eq!(
                stream.read_8bytes().unwrap(),
                Byte8(10, 11, 12, 13, 14, 15, 16, 17)
            );
            assert_eq!(stream.byte_offset, 8);
            assert!(stream.read_8bytes().is_err());
        }

        #[test]
        fn test_is_eof() {
            let mut stream = BitStream::new(vec![10]);
            assert!(!stream.is_eof());
            stream.byte_offset += 1;
            assert!(stream.is_eof());
        }

        #[test]
        fn test_skip_bytes() {
            let mut stream = BitStream::new(vec![10, 11, 12]);
            assert_eq!(stream.skip_bytes(3).unwrap(), 3);
            assert!(stream.skip_bytes(1).is_err());
        }

        #[test]
        fn test_current_byte() {
            let mut stream = BitStream::new(vec![10]);
            assert_eq!(stream.current_byte().unwrap(), 10);
            stream.byte_offset += 1;
            assert!(stream.current_byte().is_err());
        }

        #[test]
        fn test_read_bits() {
            let mut stream = BitStream::new(vec![15, 16, 17]); // 0F 00001111
            assert_eq!(stream.read_bits(5).unwrap(), 1); // 00001
            assert_eq!(stream.bit_offset, 5);
            assert_eq!(stream.read_bits(3).unwrap(), 7); // 111
            assert_eq!(stream.bit_offset, 0);
            assert_eq!(stream.byte_offset, 1);
            // A0 A1  00010000 00010001
            assert_eq!(stream.read_bits(3).unwrap(), 0); // 000
            assert_eq!(stream.bit_offset, 3);
            assert_eq!(stream.read_bits(10).unwrap(), 514); // 10000 00010
            assert_eq!(stream.bit_offset, 5);
            assert_eq!(stream.byte_offset, 2);
        }

        #[test]
        fn test_read_zero_term_string() {
            let mut stream = BitStream::new(vec![
                's' as u8, 't' as u8, 'r' as u8, 'i' as u8, 'n' as u8, 'g' as u8, 0,
            ]);
            assert_eq!(stream.read_zero_term_string(), "string");
        }

        #[test]
        fn test_extract() {
            let mut stream = BitStream::new(vec![0, 1, 2, 3, 4]);
            let ex = stream.extract(3).unwrap();
            assert_eq!(ex.inner, [0, 1, 2]);
            let ex = stream.extract(3);
            assert!(ex.is_err());
        }
    }
}
