use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};

struct BitStream {
    file: File,
    bit_offset: u8,
    byte_offset: u64,
    len: u64,
}

impl BitStream {
    fn new(file: File) -> Result<BitStream> {
        let len = file.metadata()?.len();
        Ok(BitStream {
            file: file,
            bit_offset: 0,
            byte_offset: 0,
            len: len,
        })
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut ch = [0; 1];
        self.file.read_exact(&mut ch)?;
        self.byte_offset += 1;
        Ok(ch[0])
    }

    fn is_eof(&self) -> bool {
        self.byte_offset >= self.len
    }

    fn skip_bytes(&mut self, n: i64) -> std::io::Result<u64> {
        let b = self.byte_offset as i64 + n;
        self.byte_offset = b as u64;
        self.file.seek(SeekFrom::Current(n))
    }

    // read byte without seeking
    fn current_byte(&mut self) -> Result<u8> {
        let c = self.read_byte()?;
        self.skip_bytes(-1)?;
        Ok(c)
    }

    fn read_bits(&mut self, n: u8) -> Result<u32> {
        if n == 0 {
            return Ok(0);
        }
        let mut return_bits = 0u32;
        let num_bits_left_in_byte = 8 - self.bit_offset;
        if num_bits_left_in_byte >= n {
            return_bits = (self.current_byte()? as u32 >> (num_bits_left_in_byte - n)) & ((1 << n) - 1);
            self.bit_offset += n;
        } else {
            let mut num_bits_togo = n - num_bits_left_in_byte;
            return_bits = self.current_byte()? as u32 & ((1 << num_bits_left_in_byte) - 1);
            self.skip_bytes(1)?;
            self.bit_offset = 0;
            while (num_bits_togo > 0) {
                if (num_bits_togo > 8) {
                    return_bits = (return_bits << 8) | self.current_byte()? as u32;
                    self.skip_bytes(1);
                    num_bits_togo -= 8;
                } else {
                    return_bits = (return_bits << num_bits_togo)
                        | ((self.current_byte()? as u32 >> (8 - num_bits_togo)) & ((1 << num_bits_togo) - 1));
                    self.bit_offset += num_bits_togo;
                    num_bits_togo = 0;
                }
            }
        }
        if self.bit_offset == 8 {
            self.skip_bytes(1);
            self.bit_offset = 0;
        }
        Ok(return_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_stream() -> BitStream {
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
        assert!(stream.read_byte().is_err());
    }

    #[test]
    fn test_skip_bytes() {
        let mut stream = new_stream();
        assert_eq!(stream.skip_bytes(3).unwrap(), 3);
        assert_eq!(stream.read_byte().unwrap(), 3);
        assert_eq!(stream.byte_offset, 4);
        stream.skip_bytes(28);
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
}
