use crate::traits::AccountSize;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

/// Wrapper over an unsigned number that serializes to a variable-length form.
///
/// ## Encoding
///
/// If `bit_len(number) <= 13`: flag encoding
/// else: length encoding
///
/// ### Flag encoding
///
/// Numbers are encoded in little-endian format using the first bit of each byte to indicate
/// whether the next byte is part of the number or not.
///
/// ### Length encoding
///
/// Numbers are encoded in little-endian format using the first byte to indicate the number of
/// bytes in the number.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FnkUInt(pub u64);

impl FnkUInt {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: u64) -> Self {
        Self(inner)
    }

    // GETTERS ----------------------------------------------------------------

    pub fn get_u8(&self) -> Option<u8> {
        let max = u8::MAX as u64;
        if self.0 <= max {
            Some(self.0 as u8)
        } else {
            None
        }
    }

    pub fn get_u16(&self) -> Option<u16> {
        let max = u16::MAX as u64;
        if self.0 <= max {
            Some(self.0 as u16)
        } else {
            None
        }
    }

    pub fn get_u32(&self) -> Option<u32> {
        let max = u32::MAX as u64;
        if self.0 <= max {
            Some(self.0 as u32)
        } else {
            None
        }
    }

    pub fn get_u64(&self) -> u64 {
        self.0
    }

    pub fn get_usize(&self) -> Option<usize> {
        let max = usize::MAX as u64;
        if self.0 <= max {
            Some(self.0 as usize)
        } else {
            None
        }
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> u64 {
        self.0
    }
}

impl AsRef<u64> for FnkUInt {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Deref for FnkUInt {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FnkUInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for FnkUInt {
    fn from(v: u8) -> Self {
        Self(v as u64)
    }
}

impl From<u16> for FnkUInt {
    fn from(v: u16) -> Self {
        Self(v as u64)
    }
}

impl From<u32> for FnkUInt {
    fn from(v: u32) -> Self {
        Self(v as u64)
    }
}

impl From<u64> for FnkUInt {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<usize> for FnkUInt {
    fn from(v: usize) -> Self {
        Self(v as u64)
    }
}

impl BorshSerialize for FnkUInt {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let bit_length = 64 - self.0.leading_zeros();

        if bit_length <= 13 {
            // Flag encoding.
            let byte_length = if bit_length <= 6 {
                1
            } else {
                (bit_length - 6 + 8) / 8 + 1
            };

            // Write first.
            let mut byte = (self.0 & 0x3F) as u8;

            // Include next flag.
            if byte_length > 1 {
                byte |= 0x40;
            }

            writer.write_all(&[byte])?;

            // Write remaining bytes.
            let mut offset = 6;
            let last = byte_length - 1;
            for i in 1..byte_length {
                let mut byte = ((self.0 >> offset) & 0x7F) as u8 | 0x80;

                if i >= last {
                    byte &= 0x7F;
                }

                writer.write_all(&[byte])?;
                offset += 7;
            }
        } else {
            // Length encoding.
            let byte_length = ((bit_length + 8) / 8).min(8);
            let bytes = self.0.to_le_bytes();
            let bytes = &bytes.as_slice()[..byte_length as usize];
            let byte_length = byte_length as u8 | 0x80;

            writer.write_all(&[byte_length as u8])?;
            writer.write_all(bytes)?;
        }

        Ok(())
    }
}

impl BorshDeserialize for FnkUInt {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        if buf.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Unexpected length of input",
            ));
        }

        let first_byte = buf[0];

        if first_byte & 0x80 == 0 {
            // Flag encoding.
            let mut number = (first_byte & 0x3F) as u64;
            *buf = &buf[1..];

            if first_byte & 0x40 != 0 {
                // Read remaining bytes.
                let mut offset = 6;

                loop {
                    if buf.is_empty() {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            "Unexpected length of input",
                        ));
                    }

                    let byte = buf[0];
                    *buf = &buf[1..];

                    number |= ((byte & 0x7F) as u64) << offset;

                    if (byte & 0x80) == 0 {
                        break;
                    }

                    offset += 7;
                }
            }

            Ok(Self(number))
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x7F;

            if buf.len() < byte_length as usize + 1 {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Unexpected length of input",
                ));
            }

            let mut number = 0;

            let mut offset = 0;
            for i in 0..byte_length {
                let byte = (buf[i as usize + 1] as u64) << offset;
                number |= byte;
                offset += 8;
            }

            *buf = &buf[byte_length as usize + 1..];
            Ok(Self(number))
        }
    }
}

impl AccountSize for FnkUInt {
    fn min_account_size() -> usize {
        1
    }

    fn actual_account_size(&self) -> usize {
        let bit_length = 64 - self.0.leading_zeros();

        if bit_length <= 13 {
            // Flag encoding.
            let byte_length = if bit_length <= 6 {
                1
            } else {
                (bit_length - 6 + 8) / 8 + 1
            };

            byte_length as usize
        } else {
            // Length encoding.
            let byte_length = ((bit_length + 8) / 8).min(8);

            (byte_length + 1) as usize
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_get_from() {
        for number in [0u8, 50, u8::MAX] {
            let fnk_number = FnkUInt::from(number);
            assert_eq!(fnk_number.get_u8(), Some(number));
        }

        for number in [0u16, 50, u16::MAX] {
            let fnk_number = FnkUInt::from(number);
            assert_eq!(fnk_number.get_u16(), Some(number));
        }

        for number in [0u32, 50, u32::MAX] {
            let fnk_number = FnkUInt::from(number);
            assert_eq!(fnk_number.get_u32(), Some(number));
        }

        for number in [0usize, 50, usize::MAX] {
            let fnk_number = FnkUInt::from(number);
            assert_eq!(fnk_number.get_usize(), Some(number));
        }
    }

    #[test]
    fn test_serialize_as_one_byte_flag_format() {
        for number in [0u8, 1, 42, 63] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkUInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            assert_eq!(buffer, vec![number]);
        }
    }

    #[test]
    fn test_serialize_as_two_bytes_flag_format() {
        let number = 0b0001_0101_0101_0101u64; // 5461
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkUInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer, vec![0b0101_0101, 0b0101_0101]);
    }

    #[test]
    fn test_serialize_as_two_bytes_length_format() {
        let number = 0x2AAAu64;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkUInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer, vec![2u8 | 0x80, 0b1010_1010, 0b10_1010]);
    }

    #[test]
    fn test_serialize_as_bytes_length_format() {
        let mut number = 0x1AAu64;
        for i in 3u8..9 {
            number = (number << 8) | 0xAA;

            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkUInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", i));

            let mut result = vec![i | 0x80];
            result.resize(i as usize, 0b1010_1010);
            result.push(0b1);

            assert_eq!(buffer, result, "Incorrect result for {}", i);
        }

        let number = u64::MAX;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkUInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        let mut result = vec![8u8 | 0x80];
        result.resize(9, 0b1111_1111);

        assert_eq!(buffer, result, "Incorrect result for max");
    }

    #[test]
    fn test_deserialize() {
        for number in [
            0u64,
            1,
            u8::MAX as u64,
            u16::MAX as u64,
            u32::MAX as u64,
            usize::MAX as u64,
            u64::MAX,
        ] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkUInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let mut de_buf = buffer.as_slice();
            let deserialized = FnkUInt::deserialize(&mut de_buf)
                .unwrap_or_else(|_| panic!("Failed to deserialize for {}", number));

            assert_eq!(
                deserialized.get_u64(),
                number,
                "Incorrect result for {}",
                number
            );
            assert!(de_buf.is_empty(), "Buffer not empty for {}", number);
        }
    }

    #[test]
    fn test_deserialize_long() {
        for number in 64u64..1000 {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkUInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let mut de_buf = buffer.as_slice();
            let deserialized = FnkUInt::deserialize(&mut de_buf)
                .unwrap_or_else(|_| panic!("Failed to deserialize for {}", number));

            assert_eq!(
                deserialized.get_u64(),
                number,
                "Incorrect result for {}",
                number
            );
            assert!(de_buf.is_empty(), "Buffer not empty for {}", number);
        }
    }
}
