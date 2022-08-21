use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

const MIN_I64_ABS: u64 = 9223372036854775808;

/// Wrapper over a signed number that serializes to a variable-length form.
///
/// ## Encoding
///
/// If `bit_len(abs(number)) <= 12`: flag encoding
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
        let bit_length = 64 - number.leading_zeros();

        if bit_length <= 12 {
            // Flag encoding.
            let byte_length = if bit_length <= 5 {
                1
            } else {
                (bit_length - 5 + 8) / 8 + 1
            };

            // Write first.
            let mut byte = (number & 0x1F) as u8;

            // Include next flag.
            if byte_length > 1 {
                byte |= 0x40;
            }

            // Negative flag.
            if self.0 < 0 {
                byte |= 0x20;
            }

            writer.write_all(&[byte])?;

            // Write remaining bytes.
            let mut offset = 5;
            let last = byte_length - 1;
            for i in 1..byte_length {
                let mut byte = ((number >> offset) & 0x7F) as u8 | 0x80;

                if i >= last {
                    byte &= 0x7F;
                }

                writer.write_all(&[byte])?;
                offset += 7;
            }
        } else {
            // Length encoding.
            let byte_length = ((bit_length + 8) / 8).min(8);
            let bytes = number.to_le_bytes();
            let bytes = &bytes.as_slice()[..byte_length as usize];
            let mut byte_length = byte_length as u8 | 0x80;

            // Negative flag.
            if self.0 < 0 {
                byte_length |= 0x40;
            }

            writer.write_all(&[byte_length as u8])?;
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
                // Read remaining bytes.
                let mut offset = 5;

                loop {
                    if buf.is_empty() {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            "Unexpected length of input",
                        ));
                    }

                    let byte = buf[0];
                    *buf = &buf[1..];

                    number |= ((byte & 0x7F) as i64) << offset;

                    if (byte & 0x80) == 0 {
                        break;
                    }

                    offset += 7;
                }
            }

            // Negative.
            if first_byte & 0x20 != 0 {
                Ok(Self(-number))
            } else {
                Ok(Self(number))
            }
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x3F;

            if buf.len() < byte_length as usize + 1 {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Unexpected length of input",
                ));
            }

            let mut number: u64 = 0;

            let mut offset = 0;
            for i in 0..byte_length {
                let byte = (buf[i as usize + 1] as u64) << offset;
                number |= byte;
                offset += 8;
            }

            *buf = &buf[byte_length as usize + 1..];

            // Negative.
            if first_byte & 0x40 != 0 {
                if number == MIN_I64_ABS {
                    Ok(Self(i64::MIN))
                } else if number >= i64::MAX as u64 {
                    Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Number underflow",
                    ))
                } else {
                    Ok(Self(-(number as i64)))
                }
            } else {
                if number > i64::MAX as u64 {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Number overflow",
                    ));
                }

                Ok(Self(number as i64))
            }
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
    fn test_serialize_as_one_byte_flag_format() {
        for number in [0i8, 1, 15, 31] {
            // Positive
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            assert_eq!(
                buffer,
                vec![number as u8],
                "Incorrect serialization for {}",
                number
            );

            if number == 0 {
                continue;
            }

            // Negative
            let number = -number;
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", number));

            assert_eq!(
                buffer,
                vec![number.unsigned_abs() | 0x20],
                "Incorrect serialization for {}",
                number
            );
        }
    }

    #[test]
    fn test_serialize_as_two_bytes_flag_format() {
        // Positive
        let number = 0b1010_1010_1010i64; // 2730
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer, vec![0b0100_1010, 0b0101_0101]);

        // Negative
        let number = -0b1010_1010_1010i64; // -2730
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer, vec![0b0110_1010, 0b0101_0101]);
    }

    #[test]
    fn test_serialize_as_two_bytes_length_format() {
        // Positive
        let number = 0x1555i64;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize positive");

        assert_eq!(
            buffer,
            vec![2u8 | 0x80, 0b0101_0101, 0b1_0101],
            "Incorrect positive serialization"
        );

        // Negative
        let number = -0x1555i64;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize negative");

        assert_eq!(
            buffer,
            vec![2u8 | 0x80 | 0x40, 0b0101_0101, 0b1_0101],
            "Incorrect negative serialization"
        );
    }

    #[test]
    fn test_serialize_as_bytes_length_format() {
        // Positive
        let mut number = 0x1AAi64;
        for i in 3u8..9 {
            number = (number << 8) | 0xAA;

            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", i));

            let mut result = vec![i | 0x80];
            result.resize(i as usize, 0b1010_1010);
            result.push(0b1);

            assert_eq!(buffer, result, "Incorrect result for {}", i);
        }

        let number = i64::MAX;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        let mut result = vec![8u8 | 0x80];
        result.resize(8, 0b1111_1111);
        result.push(0b0111_1111);

        assert_eq!(buffer, result, "Incorrect result for max");

        // Negative
        let mut number = 0x1AAi64;
        for i in 3u8..9 {
            number = (number << 8) | 0xAA;

            let number = -number;
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkInt::from(number);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", -(i as i8)));

            let mut result = vec![i | 0x80 | 0x40];
            result.resize(i as usize, 0b1010_1010);
            result.push(0b1);

            assert_eq!(buffer, result, "Incorrect result for {}", -(i as i8));
        }

        let number = i64::MIN + 1;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        let mut result = vec![8u8 | 0x80 | 0x40];
        result.resize(8, 0b1111_1111);
        result.push(0b0111_1111);

        assert_eq!(buffer, result, "Incorrect result for min -1");

        let number = i64::MIN;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkInt::from(number);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        let mut result = vec![8u8 | 0x80 | 0x40];
        result.resize(8, 0b0);
        result.push(0b01000_0000);

        assert_eq!(buffer, result, "Incorrect result for min");
    }

    #[test]
    fn test_deserialize() {
        for number in [
            0i64,
            1,
            u8::MAX as i64,
            u16::MAX as i64,
            u32::MAX as i64,
            i8::MIN as i64,
            i8::MAX as i64,
            i16::MIN as i64,
            i16::MAX as i64,
            i32::MIN as i64,
            i32::MAX as i64,
            i64::MIN as i64,
            i64::MAX as i64,
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

    #[test]
    fn test_deserialize_long() {
        for number in -1000i64..1000 {
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
