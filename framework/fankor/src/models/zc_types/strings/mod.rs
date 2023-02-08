pub use fnk::*;
use std::mem::size_of;

mod fnk;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{CopyType, ZeroCopyType};
use crate::traits::AccountSize;
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

pub struct ZcString<'info> {
    info: &'info AccountInfo<'info>,
    offset: usize,
}

impl<'info> ZeroCopyType<'info> for ZcString<'info> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((ZcString { info, offset }, None))
    }

    fn read_byte_size_from_bytes(mut bytes: &[u8]) -> FankorResult<usize> {
        let length = u32::deserialize(&mut bytes)?;
        Ok(length as usize + size_of::<u32>())
    }
}

impl<'info> CopyType<'info> for String {
    type ZeroCopyType = ZcString<'info>;

    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }
}

impl<'info> ZcString<'info> {
    // GETTERS ----------------------------------------------------------------

    pub fn len(&self) -> FankorResult<usize> {
        let bytes =
            self.info
                .try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: "ZcString",
                })?;
        let mut bytes = &bytes[self.offset..];
        let length = u32::deserialize(&mut bytes)?;

        Ok(length as usize)
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
                    type_name: "ZcString",
                })?;
        let mut bytes = &bytes[self.offset..];
        let length = u32::deserialize(&mut bytes)?;
        let size = length as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "String",
            }
            .into());
        }

        let text = std::str::from_utf8(&bytes[..size]).map_err(|_| {
            FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: "String",
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
                    type_name: "ZcString",
                })?;
        let mut bytes = &bytes[self.offset..];
        let length = u32::deserialize(&mut bytes)?;
        let size = length as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "String",
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
    use super::*;

    #[test]
    fn test_read_byte_length() {
        let vector = vec![5, 0, 0, 0, 1, 2, 3, 4, 5, 99, 99, 99];
        let size = ZcString::read_byte_size_from_bytes(&vector).unwrap();

        assert_eq!(size, size_of::<u32>() + 5 * size_of::<u8>());
    }
}
