use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::prelude::{FnkString, FnkUInt};
use crate::traits::{CopyType, ZeroCopyType};

pub struct ZcFnkString<'info> {
    info: &'info AccountInfo<'info>,
    offset: usize,
}

impl<'info> ZeroCopyType<'info> for ZcFnkString<'info> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((ZcFnkString { info, offset }, None))
    }

    fn read_byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let bytes = &mut bytes;
        let initial_len = bytes.len();
        let length = FnkUInt::deserialize(bytes)?;
        let length_field_size = initial_len - bytes.len();

        Ok(length
            .get_usize()
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?
            + length_field_size)
    }
}

impl<'info, 'a> CopyType<'info> for FnkString<'a> {
    type ZeroCopyType = ZcFnkString<'info>;

    fn byte_size(&self) -> usize {
        let length = FnkUInt::from(self.0.len() as u64);
        length.byte_size() + self.0.len()
    }

    fn min_byte_size() -> usize {
        FnkUInt::min_byte_size()
    }
}

impl<'info> ZcFnkString<'info> {
    // GETTERS ----------------------------------------------------------------

    pub fn len(&self) -> FankorResult<usize> {
        let bytes =
            self.info
                .try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: "ZcFnkString",
                })?;
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;

        Ok(length
            .get_usize()
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?)
    }

    pub fn is_empty(&self) -> FankorResult<bool> {
        Ok(self.len()? == 0)
    }

    // METHODS ----------------------------------------------------------------

    /// Reads the string as `&str` without copying it.
    pub fn borrow_as_str<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes =
            self.info
                .try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: "ZcFnkString",
                })?;
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;
        let size = length
            .get_usize()
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkString",
            }
                .into());
        }

        let text = std::str::from_utf8(&bytes[..size]).map_err(|_| {
            FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: "FnkString",
            }
        })?;

        Ok(f(text))
    }

    /// Reads the string as `&str` without copying it.
    ///
    /// # Safety
    /// Differs from `borrow_as_str` in that this method returns does not check
    /// the string is a valid UTF-8 string.
    pub fn borrow_as_str_unchecked<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes =
            self.info
                .try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: "ZcFnkString",
                })?;
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;
        let size = length
            .get_usize()
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkString",
            }
                .into());
        }

        let text = unsafe { std::str::from_utf8_unchecked(&bytes[..size]) };

        Ok(f(text))
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_read_byte_length() {
        let vector = vec![5, 1, 2, 3, 4, 5, 99, 99, 99];
        let size = ZcFnkString::read_byte_size(&vector).unwrap();

        assert_eq!(size, 1 + 5 * size_of::<u8>());
    }
}
