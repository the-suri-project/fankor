use crate::errors::{FankorErrorCode, FankorResult};
use crate::prelude::{FnkInt, FnkUInt};
use crate::traits::{CopyType, ZeroCopyType};
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

impl<'info> ZeroCopyType<'info> for FnkInt {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &bytes[offset..];
        let initial_size = bytes.len();
        let value = FnkInt::deserialize(&mut bytes)?;

        Ok((value, Some(initial_size - bytes.len())))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkInt",
            }
            .into());
        }

        let first_byte = bytes[0];
        let result = if first_byte & 0x80 == 0 {
            // Flag encoding.
            let mut size = 1;

            if first_byte & 0x40 != 0 {
                if bytes.is_empty() {
                    return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                        type_name: "FnkInt",
                    }
                    .into());
                }

                size += 1;
            }

            size
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x3F;

            if byte_length >= 7 {
                return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                    type_name: "FnkInt",
                }
                .into());
            }

            let byte_length = byte_length as usize + 2;

            if bytes.len() < byte_length + 1 {
                return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                    type_name: "FnkInt",
                }
                .into());
            }

            byte_length + 1
        };

        Ok(result)
    }
}

impl<'info> CopyType<'info> for FnkInt {
    type ZeroCopyType = FnkInt;

    fn byte_size(&self) -> usize {
        const FLAG_ENCODING_LIMIT: u64 = 1 << 13; // 2^13
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

    fn min_byte_size() -> usize {
        1
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info> ZeroCopyType<'info> for FnkUInt {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &bytes[offset..];
        let initial_size = bytes.len();
        let value = FnkUInt::deserialize(&mut bytes)?;

        Ok((value, Some(initial_size - bytes.len())))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkUInt",
            }
            .into());
        }

        let first_byte = bytes[0];
        let result = if first_byte & 0x80 == 0 {
            // Flag encoding.
            let mut size = 1;

            if first_byte & 0x40 != 0 {
                if bytes.is_empty() {
                    return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                        type_name: "FnkUInt",
                    }
                    .into());
                }

                size += 1;
            }

            size
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x7F;

            if byte_length >= 7 {
                return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                    type_name: "FnkUInt",
                }
                .into());
            }

            let byte_length = byte_length as usize + 2;

            if bytes.len() < byte_length + 1 {
                return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                    type_name: "FnkUInt",
                }
                .into());
            }

            byte_length + 1
        };

        Ok(result)
    }
}

impl<'info> CopyType<'info> for FnkUInt {
    type ZeroCopyType = FnkUInt;

    fn byte_size(&self) -> usize {
        const FLAG_ENCODING_LIMIT: u64 = 1 << 14; // 2^14
        if self.0 < FLAG_ENCODING_LIMIT {
            // Flag encoding.
            if self.0 >> 6 != 0 {
                2
            } else {
                1
            }
        } else {
            // Length encoding.
            let mut byte_length = 9; // 8 bytes + 1 byte for length.
            let bytes = self.0.to_le_bytes();

            for i in (1..8).rev() {
                if bytes[i] != 0 {
                    break;
                }

                byte_length -= 1;
            }

            byte_length
        }
    }

    fn min_byte_size() -> usize {
        1
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use borsh::BorshSerialize;
    use std::io::Cursor;

    #[test]
    fn test_signed_read_byte_size() {
        for number in [
            0i64,
            1,
            -1,
            (1 << 5) - 1,
            -((1 << 5) - 1),
            1 << 5,
            -(1 << 5),
            (1 << 14) - 1,
            -((1 << 14) - 1),
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

            assert_eq!(
                FnkInt::read_byte_size(&buffer).expect("Cannot read byte size"),
                fnk_number.byte_size(),
                "Incorrect result for {}",
                number
            );
        }
    }

    #[test]
    fn test_unsigned_read_byte_size() {
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

            assert_eq!(
                FnkUInt::read_byte_size(&buffer).expect("Cannot read byte size"),
                fnk_number.byte_size(),
                "Incorrect result for {}",
                number
            );
        }
    }
}
