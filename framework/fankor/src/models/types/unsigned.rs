use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Display;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

const FLAG_ENCODING_LIMIT: u64 = 1 << 14; // 2^14

/// Wrapper over an unsigned number that serializes to a variable-length form.
///
/// ## Encoding
///
/// If the number is less than 2^14, the flag encoding is applied, otherwise the length encoding
/// is used. The encoding used is determined by the HSB of the first encoded byte so:
///
/// ```none
/// Flag encoding  : 0___ ____
/// Length encoding: 1___ ____
/// ```
///
/// ### Flag encoding
///
/// Numbers are encoded in little-endian format using the second bit of the first byte to indicate
/// whether the next byte is part of the number (1) or not (0).
///
/// ```none
/// 1 byte : 00nn nnnn           -> Big endian: 00nn nnnn
/// 2 bytes: 01nn nnnn mmmm mmmm -> Big endian: 00mm mmmm mmnn nnnn
/// no more cases
/// ```
///
/// ### Length encoding
///
/// Numbers are encoded in little-endian format using the first byte to indicate the actual
/// length in bytes of the number. The length must be in range [0, 6] which actually represents
/// the range [2, 8], other values are forbidden.
///
/// ```none
/// 1000 0sss + sss bytes
/// ```
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

impl Display for FnkUInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
        if self.0 < FLAG_ENCODING_LIMIT {
            // Flag encoding.
            let mut remaining = self.0;

            // Write first.
            let mut byte = (remaining & 0x3F) as u8;
            remaining >>= 6;

            // Include next flag.
            if remaining != 0 {
                byte |= 0x40;
            }

            writer.write_all(&[byte])?;

            // Write second byte.
            if remaining != 0 {
                byte = remaining as u8;
                writer.write_all(&[byte])?;
            }
        } else {
            // Length encoding.
            let mut byte_length = 8;
            let bytes = self.0.to_le_bytes();

            for i in (1..8).rev() {
                if bytes[i] != 0 {
                    break;
                }

                byte_length -= 1;
            }

            debug_assert!((2i32..=8).contains(&byte_length), "Invalid byte length");

            let bytes = &bytes.as_slice()[..byte_length as usize];
            let byte_length = (byte_length - 2) as u8 | 0x80;

            writer.write_all(&[byte_length])?;
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
                // Read second byte.
                if buf.is_empty() {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Unexpected length of input",
                    ));
                }

                let byte = buf[0];
                *buf = &buf[1..];

                number |= (byte as u64) << 6;
            }

            Ok(Self(number))
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x7F;

            if byte_length >= 7 {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Incorrect FnkUInt length",
                ));
            }

            let byte_length = byte_length as usize + 2;

            if buf.len() < byte_length + 1 {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Unexpected length of input",
                ));
            }

            let mut number = 0;
            let mut offset = 0;

            for i in 0..byte_length {
                number |= (buf[i + 1] as u64) << offset;
                offset += 8;
            }

            *buf = &buf[byte_length + 1..];
            Ok(Self(number))
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::CopyType;
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

        let fnk_number = FnkUInt::from(u8::MAX as u64 + 1);
        assert_eq!(fnk_number.get_u8(), None);

        let fnk_number = FnkUInt::from(u16::MAX as u64 + 1);
        assert_eq!(fnk_number.get_u8(), None);
        assert_eq!(fnk_number.get_u16(), None);

        let fnk_number = FnkUInt::from(u32::MAX as u64 + 1);
        assert_eq!(fnk_number.get_u8(), None);
        assert_eq!(fnk_number.get_u16(), None);
        assert_eq!(fnk_number.get_u32(), None);
    }

    #[test]
    fn test_serialize_flag_format_one_byte() {
        let max = 1u8 << 6;
        for number in 0..max {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkUInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            assert_eq!(buffer, vec![number]);
            assert_eq!(fnk_number.byte_size(), 1);
        }
    }

    #[test]
    fn test_serialize_flag_format_two_bytes() {
        let min = 1 << 6;
        for number in min..FLAG_ENCODING_LIMIT {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkUInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let first_byte = (0x40 | number & 0x3F) as u8;
            let second_byte = (number >> 6) as u8;
            assert_eq!(buffer.len(), 2);
            assert_eq!(buffer[0], first_byte);
            assert_eq!(buffer[1], second_byte);
            assert_eq!(fnk_number.byte_size(), 2);
        }
    }

    #[test]
    fn test_serialize_length_format() {
        // Three bytes
        let num_bytes = 2;
        for number in [1u64 << 14, u16::MAX as u64] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkUInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let length = (0x80 | (num_bytes - 2)) as u8;
            assert_eq!(buffer.len(), num_bytes + 1);
            assert_eq!(buffer[0], length);
            assert_eq!(&buffer[1..], &number.to_le_bytes()[..num_bytes]);
            assert_eq!(fnk_number.byte_size(), num_bytes + 1);
        }

        // Rest
        for num_bytes in 3..=8 {
            let low = 1u64 << ((num_bytes - 1) << 3);
            let high = ((1u128 << (num_bytes << 3)) - 1) as u64;

            for number in [low, high] {
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                let fnk_number = FnkUInt::from(number);
                fnk_number
                    .serialize(&mut cursor)
                    .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

                let length = (0x80 | (num_bytes - 2)) as u8;
                assert_eq!(buffer.len(), num_bytes + 1);
                assert_eq!(buffer[0], length);
                assert_eq!(&buffer[1..], &number.to_le_bytes()[..num_bytes]);
                assert_eq!(fnk_number.byte_size(), num_bytes + 1);
            }
        }
    }

    #[test]
    fn test_deserialize() {
        for number in [
            0u64,
            1,
            (1 << 6) - 1,
            1 << 6,
            (1 << 14) - 1,
            1 << 14,
            1 << 16,
            1 << 24,
            1 << 32,
            1 << 40,
            1 << 48,
            1 << 56,
            u8::MAX as u64,
            u16::MAX as u64,
            u32::MAX as u64,
            usize::MAX as u64,
            u64::MAX / 2,
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
}
