use crate::traits::AccountSize;
use borsh::{BorshDeserialize, BorshSerialize};
use std::cmp::Ordering;
use std::fmt::Display;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

const FLAG_ENCODING_LIMIT: u64 = 1 << 13; // 2^13
const MIN_I64_ABS: u64 = i64::MIN.unsigned_abs();

/// Wrapper over a signed number that serializes to a variable-length form.
///
/// ## Encoding
///
/// If the `abs(number)` is less than 2^13, the flag encoding is applied, otherwise the length
/// encoding is used. The encoding used is determined by the HSB of the first encoded byte so:
///
/// ```none
/// Flag encoding  : 0___ ____
/// Length encoding: 1___ ____
/// ```
///
/// Both encodings compute the absolute value of the number and store the sign bit separately,
/// being 0 for positive numbers and 1 for negative numbers.
///
/// ### Flag encoding
///
/// Numbers are encoded in little-endian format using the first bit of each byte to indicate
/// whether the next byte is part of the number (1) or not (0).
///
/// Positive numbers:
/// ```none
/// 1 byte : 000n nnnn           -> Big endian: 000n nnnn
/// 2 bytes: 010n nnnn mmmm mmmm -> Big endian: 000m mmmm mmmn nnnn
/// no more cases
/// ```
///
/// Negative numbers:
/// ```none
/// 1 byte : 001n nnnn           -> Big endian: -(000n nnnn)
/// 2 bytes: 011n nnnn mmmm mmmm -> Big endian: -(000m mmmm mmmn nnnn)
/// no more cases
/// ```
///
/// ### Length encoding
///
/// Numbers are encoded in little-endian format using the first byte to indicate the actual
/// length in bytes of the number as well as the sign bit (second HSB). The length must be
/// in range [0, 6] which actually represents the range [2, 8], other values are forbidden.
///
/// ```none
/// 1S00 0sss + sss bytes
///  \_ sign bit
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FnkInt(pub i64);

impl FnkInt {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: i64) -> Self {
        Self(inner)
    }

    // GETTERS ----------------------------------------------------------------

    pub fn get_u8(&self) -> Option<u8> {
        let max = u8::MAX as i64;
        if 0 <= self.0 && self.0 <= max {
            Some(self.0 as u8)
        } else {
            None
        }
    }

    pub fn get_u16(&self) -> Option<u16> {
        let max = u16::MAX as i64;
        if 0 <= self.0 && self.0 <= max {
            Some(self.0 as u16)
        } else {
            None
        }
    }

    pub fn get_u32(&self) -> Option<u32> {
        let max = u32::MAX as i64;
        if 0 <= self.0 && self.0 <= max {
            Some(self.0 as u32)
        } else {
            None
        }
    }

    pub fn get_u64(&self) -> Option<u64> {
        let max = i64::MAX;
        if 0 <= self.0 && self.0 <= max {
            Some(self.0 as u64)
        } else {
            None
        }
    }

    pub fn get_usize(&self) -> Option<usize> {
        #[cfg(target_pointer_width = "64")]
        return self.get_u64().map(|x| x as usize);

        #[cfg(target_pointer_width = "32")]
        return self.get_u32().map(|x| x as usize);
    }

    pub fn get_i8(&self) -> Option<i8> {
        let min = i8::MIN as i64;
        let max = i8::MAX as i64;
        if min <= self.0 && self.0 <= max {
            Some(self.0 as i8)
        } else {
            None
        }
    }

    pub fn get_i16(&self) -> Option<i16> {
        let min = i16::MIN as i64;
        let max = i16::MAX as i64;
        if min <= self.0 && self.0 <= max {
            Some(self.0 as i16)
        } else {
            None
        }
    }

    pub fn get_i32(&self) -> Option<i32> {
        let min = i32::MIN as i64;
        let max = i32::MAX as i64;
        if min <= self.0 && self.0 <= max {
            Some(self.0 as i32)
        } else {
            None
        }
    }

    pub fn get_i64(&self) -> i64 {
        self.0
    }

    pub fn get_isize(&self) -> Option<isize> {
        let min = isize::MIN as i64;
        let max = isize::MAX as i64;
        if min <= self.0 && self.0 <= max {
            Some(self.0 as isize)
        } else {
            None
        }
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> i64 {
        self.0
    }
}

impl AsRef<i64> for FnkInt {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Deref for FnkInt {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FnkInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for FnkInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i8> for FnkInt {
    fn from(v: i8) -> Self {
        Self(v as i64)
    }
}

impl From<i16> for FnkInt {
    fn from(v: i16) -> Self {
        Self(v as i64)
    }
}

impl From<i32> for FnkInt {
    fn from(v: i32) -> Self {
        Self(v as i64)
    }
}

impl From<i64> for FnkInt {
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<isize> for FnkInt {
    fn from(v: isize) -> Self {
        Self(v as i64)
    }
}

impl From<u8> for FnkInt {
    fn from(v: u8) -> Self {
        Self(v as i64)
    }
}

impl From<u16> for FnkInt {
    fn from(v: u16) -> Self {
        Self(v as i64)
    }
}

impl From<u32> for FnkInt {
    fn from(v: u32) -> Self {
        Self(v as i64)
    }
}

impl TryFrom<u64> for FnkInt {
    type Error = ();

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        let max = i64::MAX as u64;
        if v <= max {
            Ok(Self(v as i64))
        } else {
            Err(())
        }
    }
}

#[cfg(target_pointer_width = "32")]
impl From<usize> for FnkInt {
    fn from(v: usize) -> Self {
        Self(v as i64)
    }
}

#[cfg(target_pointer_width = "64")]
impl TryFrom<usize> for FnkInt {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        let max = i64::MAX as usize;
        if v <= max {
            Ok(Self(v as i64))
        } else {
            Err(())
        }
    }
}

impl BorshSerialize for FnkInt {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let number = self.0.unsigned_abs();

        if number < FLAG_ENCODING_LIMIT {
            // Flag encoding.
            let mut remaining = number;

            // Write first.
            let mut byte = (remaining & 0x1F) as u8;
            remaining >>= 5;

            // Include next flag.
            if remaining != 0 {
                byte |= 0x40;
            }

            // Include sign bit.
            if self.0 < 0 {
                byte |= 0x20;
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
            let bytes = number.to_le_bytes();

            for i in (1..8).rev() {
                if bytes[i] != 0 {
                    break;
                }

                byte_length -= 1;
            }

            debug_assert!((2i32..=8).contains(&byte_length), "Invalid byte length");

            let bytes = &bytes.as_slice()[..byte_length as usize];
            let mut byte_length = (byte_length - 2) as u8 | 0x80;

            // Include sign bit.
            if self.0 < 0 {
                byte_length |= 0x40;
            }

            writer.write_all(&[byte_length])?;
            writer.write_all(bytes)?;
        }

        Ok(())
    }
}

impl BorshDeserialize for FnkInt {
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
            let mut number = (first_byte & 0x1F) as i64;
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

                number |= (byte as i64) << 5;
            }

            // Process sign bit.
            if first_byte & 0x20 != 0 {
                number = -number
            }

            Ok(Self(number))
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x3F;

            if byte_length >= 7 {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Incorrect FnkInt length",
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

            let number = if first_byte & 0x40 == 0 {
                i64::try_from(number).map_err(|_| {
                    std::io::Error::new(ErrorKind::InvalidInput, "Incorrect FnkInt value")
                })?
            } else {
                match number.cmp(&MIN_I64_ABS) {
                    Ordering::Less => -(number as i64),
                    Ordering::Equal => i64::MIN,
                    Ordering::Greater => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            "Incorrect FnkInt value",
                        ));
                    }
                }
            };

            Ok(Self(number))
        }
    }
}

impl AccountSize for FnkInt {
    fn min_account_size() -> usize {
        1
    }

    fn actual_account_size(&self) -> usize {
        let number = self.0.unsigned_abs();

        if number < FLAG_ENCODING_LIMIT {
            // Flag encoding.
            if number >> 5 != 0 {
                2
            } else {
                1
            }
        } else {
            // Length encoding.
            let mut byte_length = 9; // 8 bytes + 1 byte for length.
            let bytes = number.to_le_bytes();

            for i in (1..8).rev() {
                if bytes[i] != 0 {
                    break;
                }

                byte_length -= 1;
            }

            byte_length
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
        // Unsigned
        for number in [0u8, 50, u8::MAX] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_u8(), Some(number));
        }

        for number in [0u16, 50, u16::MAX] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_u16(), Some(number));
        }

        for number in [0u32, 50, u32::MAX] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_u32(), Some(number));
        }

        #[cfg(target_pointer_width = "32")]
        for number in [0usize, 50, usize::MAX] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_usize(), Some(number));
        }

        #[cfg(target_pointer_width = "64")]
        for number in [0usize, 50, isize::MAX as usize] {
            let fnk_number = FnkInt::try_from(number).expect("Failed to convert number");
            assert_eq!(fnk_number.get_usize(), Some(number));
        }

        #[cfg(target_pointer_width = "64")]
        for number in [isize::MAX as usize + 1, usize::MAX] {
            FnkInt::try_from(number).expect_err("Conversion must fail");
        }

        for number in [0u64, 50, isize::MAX as u64] {
            let fnk_number = FnkInt::try_from(number).expect("Failed to convert number");
            assert_eq!(fnk_number.get_u64(), Some(number));
        }

        for number in [isize::MAX as u64 + 1, u64::MAX] {
            FnkInt::try_from(number).expect_err("Conversion must fail");
        }

        // Signed
        for number in [0i8, 50, -50, i8::MAX, i8::MIN] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_i8(), Some(number));
        }

        for number in [0i16, 50, -50, i16::MAX, i16::MIN] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_i16(), Some(number));
        }

        for number in [0i32, 50, -50, i32::MAX, i32::MIN] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_i32(), Some(number));
        }

        for number in [0isize, 50, -50, isize::MAX, isize::MIN] {
            let fnk_number = FnkInt::from(number);
            assert_eq!(fnk_number.get_isize(), Some(number));
        }
    }

    #[test]
    fn test_serialize_flag_format_one_byte() {
        // Positive
        let max = 1u8 << 5;
        for number in 0..max {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            assert_eq!(buffer, vec![number]);
            assert_eq!(fnk_number.actual_account_size(), 1);
        }

        // Negative
        let max = 1u8 << 5;
        for number in 1..max {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(-(number as i8));
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let result = 0x20 | number;
            assert_eq!(buffer, vec![result]);
            assert_eq!(fnk_number.actual_account_size(), 1);
        }
    }

    #[test]
    fn test_serialize_flag_format_two_bytes() {
        let min = 1 << 5;
        for number in min..FLAG_ENCODING_LIMIT as i64 {
            // Positive
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let first_byte = (0x40 | number & 0x1F) as u8;
            let second_byte = (number >> 5) as u8;
            assert_eq!(buffer.len(), 2);
            assert_eq!(buffer[0], first_byte);
            assert_eq!(buffer[1], second_byte);
            assert_eq!(fnk_number.actual_account_size(), 2);

            // Negative
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(-number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let first_byte = (0x40 | 0x20 | number & 0x1F) as u8;
            let second_byte = (number >> 5) as u8;
            assert_eq!(buffer.len(), 2);
            assert_eq!(buffer[0], first_byte);
            assert_eq!(buffer[1], second_byte);
            assert_eq!(fnk_number.actual_account_size(), 2);
        }
    }

    #[test]
    fn test_serialize_length_format() {
        // Three bytes
        let num_bytes = 2;
        for number in [1i64 << 14, u16::MAX as i64] {
            // Positive
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let length = (0x80 | (num_bytes - 2)) as u8;
            assert_eq!(buffer.len(), num_bytes + 1);
            assert_eq!(buffer[0], length);
            assert_eq!(&buffer[1..], &number.to_le_bytes()[..num_bytes]);
            assert_eq!(fnk_number.actual_account_size(), num_bytes + 1);

            // Negative
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(-number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let length = (0x80 | 0x40 | (num_bytes - 2)) as u8;
            assert_eq!(buffer.len(), num_bytes + 1);
            assert_eq!(buffer[0], length);
            assert_eq!(&buffer[1..], &number.to_le_bytes()[..num_bytes]);
            assert_eq!(fnk_number.actual_account_size(), num_bytes + 1);
        }

        // Rest until 8 bytes
        for num_bytes in 3..8 {
            let low = 1i64 << ((num_bytes - 1) << 3);
            let high = (1i64 << (num_bytes << 3)) - 1;

            for number in [low, high] {
                // Positive
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                let fnk_number = FnkInt::from(number);
                fnk_number
                    .serialize(&mut cursor)
                    .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

                let length = (0x80 | (num_bytes - 2)) as u8;
                assert_eq!(buffer.len(), num_bytes + 1);
                assert_eq!(buffer[0], length);
                assert_eq!(&buffer[1..], &number.to_le_bytes()[..num_bytes]);
                assert_eq!(fnk_number.actual_account_size(), num_bytes + 1);

                // Negative
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                let fnk_number = FnkInt::from(-number);
                fnk_number
                    .serialize(&mut cursor)
                    .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

                let length = (0x80 | 0x40 | (num_bytes - 2)) as u8;
                assert_eq!(buffer.len(), num_bytes + 1);
                assert_eq!(buffer[0], length);
                assert_eq!(&buffer[1..], &number.to_le_bytes()[..num_bytes]);
                assert_eq!(fnk_number.actual_account_size(), num_bytes + 1);
            }
        }

        // 8 bytes
        let num_bytes = 8;
        for number in [1i64 << (7 << 3), -(1i64 << (7 << 3)), i64::MIN, i64::MAX] {
            if number >= 0 {
                // Positive
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                let fnk_number = FnkInt::from(number);
                fnk_number
                    .serialize(&mut cursor)
                    .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

                let length = (0x80 | (num_bytes - 2)) as u8;
                assert_eq!(buffer.len(), num_bytes + 1);
                assert_eq!(buffer[0], length);
                assert_eq!(&buffer[1..], &number.to_le_bytes()[..num_bytes]);
                assert_eq!(fnk_number.actual_account_size(), num_bytes + 1);
            } else {
                // Negative
                let abs = number.unsigned_abs();
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                let fnk_number = FnkInt::from(number);
                fnk_number
                    .serialize(&mut cursor)
                    .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

                let length = (0x80 | 0x40 | (num_bytes - 2)) as u8;
                assert_eq!(buffer.len(), num_bytes + 1);
                assert_eq!(buffer[0], length);
                assert_eq!(&buffer[1..], &abs.to_le_bytes()[..num_bytes]);
                assert_eq!(fnk_number.actual_account_size(), num_bytes + 1);
            }
        }
    }

    #[test]
    fn test_deserialize() {
        for number in [
            0i64,
            1,
            -1,
            1 << 5 - 1,
            -(1 << 5 - 1),
            1 << 5,
            -(1 << 5),
            1 << 14 - 1,
            -(1 << 14 - 1),
            1 << 14,
            -(1 << 14),
            1 << 16,
            -(1 << 16),
            1 << 24,
            -(1 << 24),
            1 << 32,
            -(1 << 32),
            1 << 40,
            -(1 << 40),
            1 << 48,
            -(1 << 48),
            1 << 56,
            -(1 << 56),
            u8::MAX as i64,
            u16::MAX as i64,
            u32::MAX as i64,
            i8::MIN as i64,
            i8::MAX as i64,
            i16::MIN as i64,
            i16::MAX as i64,
            i32::MIN as i64,
            i32::MAX as i64,
            i64::MIN,
            i64::MIN / 2,
            i64::MAX / 2,
            i64::MAX,
            isize::MIN as i64,
            isize::MAX as i64,
        ] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            let mut de_buf = buffer.as_slice();
            let deserialized = FnkInt::deserialize(&mut de_buf)
                .unwrap_or_else(|_| panic!("Failed to deserialize for {}", number));

            assert_eq!(
                deserialized.get_i64(),
                number,
                "Incorrect result for {}",
                number
            );
            assert!(de_buf.is_empty(), "Buffer not empty for {}", number);
        }
    }
}
