use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};

#[derive(Debug, PartialEq)]
pub struct Byte2(pub u8, pub u8);

impl Byte2 {
    pub fn to_u16(&self) -> u16 {
        (u16::from(self.0) << 4) + u16::from(self.1)
    }

    pub fn to_u32(&self) -> u32 {
        u32::from(self.to_u16())
    }

    pub fn to_u64(&self) -> u64 {
        u64::from(self.to_u16())
    }
}

#[derive(Debug, PartialEq)]
pub struct Byte4(pub u8, pub u8, pub u8, pub u8);

impl Byte4 {
    pub fn to_u32(&self) -> u32 {
        (u32::from(self.0) << 12)
            + (u32::from(self.1) << 8)
            + (u32::from(self.2) << 4)
            + u32::from(self.3)
    }

    pub fn to_u64(&self) -> u64 {
        u64::from(self.to_u32())
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}{}{}{}",
            self.0 as char, self.1 as char, self.2 as char, self.3 as char
        )
    }
}

#[test]
fn test_byte4_to_string() {
    // 66 74 79 70 => ftyp
    assert_eq!(Byte4(102, 116, 121, 112).to_string(), "ftyp");
}

#[derive(Debug, PartialEq)]
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
        (u64::from(self.0) << 28)
            + (u64::from(self.1) << 24)
            + (u64::from(self.2) << 20)
            + (u64::from(self.3) << 16)
            + (u64::from(self.4) << 12)
            + (u64::from(self.5) << 8)
            + (u64::from(self.6) << 4)
            + u64::from(self.7)
    }
}

#[derive(Debug)]
pub struct BitStream {
    file: File,
    bit_offset: u8,
    byte_offset: u64,
    len: u64,
}

impl BitStream {
    pub fn new(file: File) -> Result<BitStream> {
        let len = file.metadata()?.len();
        Ok(BitStream {
            file,
            bit_offset: 0,
            byte_offset: 0,
            len,
        })
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        let mut ch = [0; 1];
        self.file.read_exact(&mut ch)?;
        self.byte_offset += 1;
        Ok(ch[0])
    }

    pub fn read_2bytes(&mut self) -> Result<Byte2> {
        let mut buf: [u8; 2] = [0; 2];
        self.file.read_exact(&mut buf)?;
        self.byte_offset += 2;
        Ok(Byte2(buf[0], buf[1]))
    }

    pub fn read_4bytes(&mut self) -> Result<Byte4> {
        let mut buf: [u8; 4] = [0; 4];
        self.file.read_exact(&mut buf)?;
        self.byte_offset += 4;
        Ok(Byte4(buf[0], buf[1], buf[2], buf[3]))
    }

    pub fn read_8bytes(&mut self) -> Result<Byte8> {
        let mut buf: [u8; 8] = [0; 8];
        self.file.read_exact(&mut buf)?;
        self.byte_offset += 8;
        Ok(Byte8(
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ))
    }

    pub fn num_bytes_left(&self) -> u64 {
        self.len - self.byte_offset
    }

    pub fn is_eof(&self) -> bool {
        self.byte_offset >= self.len
    }

    pub fn skip_bytes(&mut self, n: i64) -> Result<u64> {
        let b = self.byte_offset as i64 + n;
        self.byte_offset = b as u64;
        self.file.seek(SeekFrom::Current(n))
    }

    // read byte without seeking
    pub fn current_byte(&mut self) -> Result<u8> {
        let c = self.read_byte()?;
        self.skip_bytes(-1)?;
        Ok(c)
    }

    pub fn read_bits(&mut self, n: u32) -> Result<u32> {
        if n == 0 {
            return Ok(0);
        }
        let mut return_bits = 0u32;
        let num_bits_left_in_byte = u32::from(8 - self.bit_offset);
        if num_bits_left_in_byte >= n {
            return_bits =
                u32::from(self.current_byte()? >> (num_bits_left_in_byte - n)) & ((1 << n) - 1);
            self.bit_offset += n as u8;
        } else {
            let mut num_bits_togo = n - num_bits_left_in_byte;
            return_bits = u32::from(self.current_byte()?) & ((1 << num_bits_left_in_byte) - 1);
            self.skip_bytes(1)?;
            self.bit_offset = 0;
            while num_bits_togo > 0 {
                if num_bits_togo > 8 {
                    return_bits = (return_bits << 8) | u32::from(self.current_byte()?);
                    self.skip_bytes(1)?;
                    num_bits_togo -= 8;
                } else {
                    return_bits = (return_bits << num_bits_togo)
                        | (u32::from(self.current_byte()? >> (8 - num_bits_togo))
                            & ((1 << num_bits_togo) - 1));
                    self.bit_offset += num_bits_togo as u8;
                    num_bits_togo = 0;
                }
            }
        }
        if self.bit_offset == 8 {
            self.skip_bytes(1)?;
            self.bit_offset = 0;
        }
        Ok(return_bits)
    }

    pub fn read_zero_term_string(&mut self) -> Result<String> {
        let mut string = String::new();
        while !self.is_eof() {
            let ch = self.read_byte()?;
            if ch == 0 {
                break;
            } else {
                string.push(char::from(ch));
            }
        }
        Ok(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_stream() -> BitStream {
        // 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F
        // 10 11 12 13 14 15 16 17 18 19 1A 1B 1C 1D 1E 1F
        // 74 68 69 73 20 69 73 20 61 20 73 74 72 69 6E 67 00
        let file = File::open("./test/test.binary").unwrap();
        BitStream::new(file).unwrap()
    }

    #[test]
    fn test_read_byte() {
        let mut stream = new_stream();
        for i in 0..32 {
            assert_eq!(stream.read_byte().unwrap(), i);
            assert_eq!(stream.byte_offset, (i + 1) as u64);
        }
    }

    #[test]
    fn test_read_byte2() {
        let mut stream = new_stream();
        assert_eq!(stream.read_2bytes().unwrap(), Byte2(0, 1));
        assert_eq!(stream.byte_offset, 2);
        assert_eq!(stream.read_2bytes().unwrap(), Byte2(2, 3));
    }

    #[test]
    fn test_read_byte4() {
        let mut stream = new_stream();
        assert_eq!(stream.read_4bytes().unwrap(), Byte4(0, 1, 2, 3));
        assert_eq!(stream.byte_offset, 4);
        assert_eq!(stream.read_4bytes().unwrap(), Byte4(4, 5, 6, 7));
    }

    #[test]
    fn test_read_byte8() {
        let mut stream = new_stream();
        assert_eq!(stream.read_8bytes().unwrap(), Byte8(0, 1, 2, 3, 4, 5, 6, 7));
        assert_eq!(stream.byte_offset, 8);
        assert_eq!(
            stream.read_8bytes().unwrap(),
            Byte8(8, 9, 10, 11, 12, 13, 14, 15)
        );
    }

    #[test]
    fn test_skip_bytes() {
        let mut stream = new_stream();
        assert_eq!(stream.skip_bytes(3).unwrap(), 3);
        assert_eq!(stream.read_byte().unwrap(), 3);
        assert_eq!(stream.byte_offset, 4);
        stream.skip_bytes(45);
        assert!(stream.is_eof());
    }

    #[test]
    fn test_current_byte() {
        let mut stream = new_stream();
        assert_eq!(stream.current_byte().unwrap(), 0);
        assert_eq!(stream.byte_offset, 0);
    }

    #[test]
    fn test_read_bits() {
        let mut stream = new_stream();
        stream.skip_bytes(15); // 0F 00001111
        assert_eq!(stream.byte_offset, 15);
        assert_eq!(stream.read_bits(5).unwrap(), 1); // 00001
        assert_eq!(stream.bit_offset, 5);
        assert_eq!(stream.read_bits(3).unwrap(), 7); // 111
        assert_eq!(stream.bit_offset, 0);
        assert_eq!(stream.byte_offset, 16);
        // 10 11  00010000 00010001
        assert_eq!(stream.read_bits(3).unwrap(), 0); // 000
        assert_eq!(stream.read_bits(10).unwrap(), 514); // 10000 00010
        assert_eq!(stream.bit_offset, 5);
        assert_eq!(stream.byte_offset, 17);
    }

    #[test]
    fn test_read_zero_term_string() {
        let mut stream = new_stream();
        stream.skip_bytes(32);
        assert_eq!(stream.read_zero_term_string().unwrap(), "this is a string");
    }
}
