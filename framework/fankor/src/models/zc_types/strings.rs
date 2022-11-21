use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{ZCMut, ZeroCopyType, ZC};
use crate::prelude::{FnkString, FnkUInt};
use crate::traits::AccountSize;
use borsh::BorshDeserialize;
use std::any::type_name;

impl ZeroCopyType for String {
    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let bytes = &mut bytes;
        let initial_len = bytes.len();
        let length = u32::deserialize(bytes)?;

        Ok(length as usize + initial_len - bytes.len())
    }
}

impl<'info, 'a> ZC<'info, 'a, String> {
    // GETTERS ----------------------------------------------------------------

    pub fn len(&self) -> FankorResult<usize> {
        let bytes = self.info.data.borrow();
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
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = u32::deserialize(&mut bytes)?;
        let size = length as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8(&bytes[..size]).map_err(|_| {
            FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
        })?;

        Ok(f(text))
    }

    /// Reads the string as `&str` without copying it.
    ///
    /// # Safety
    ///
    /// Differs from `borrow_as_str` in that this method returns does not check
    /// the string is a valid UTF-8 string.
    pub unsafe fn borrow_as_str_unchecked<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = u32::deserialize(&mut bytes)?;
        let size = length as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8_unchecked(&bytes[..size]);

        Ok(f(text))
    }
}

impl<'info, 'a> ZCMut<'info, 'a, String> {
    // GETTERS ----------------------------------------------------------------

    pub fn len(&self) -> FankorResult<usize> {
        let bytes = self.info.data.borrow();
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
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = u32::deserialize(&mut bytes)?;
        let size = length as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8(&bytes[..size]).map_err(|_| {
            FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
        })?;

        Ok(f(text))
    }

    /// Reads the string as `&str` without copying it.
    ///
    /// # Safety
    ///
    /// Differs from `borrow_as_str` in that this method returns does not check
    /// the string is a valid UTF-8 string.
    pub unsafe fn borrow_as_str_unchecked<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = u32::deserialize(&mut bytes)?;
        let size = length as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8_unchecked(&bytes[..size]);

        Ok(f(text))
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info> ZeroCopyType for FnkString<'info> {
    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let bytes = &mut bytes;
        let initial_len = bytes.len();
        let length = FnkUInt::deserialize(bytes)?;

        Ok(length.0 as usize + initial_len - bytes.len())
    }
}

impl<'info, 'a> ZC<'info, 'a, FnkString<'info>> {
    // GETTERS ----------------------------------------------------------------

    pub fn len(&self) -> FankorResult<usize> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;

        Ok(length.0 as usize)
    }

    pub fn is_empty(&self) -> FankorResult<bool> {
        Ok(self.len()? == 0)
    }

    // METHODS ----------------------------------------------------------------

    /// Reads the string as `&str` without copying it.
    pub fn borrow_as_str<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;
        let size = length.0 as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8(&bytes[..size]).map_err(|_| {
            FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
        })?;

        Ok(f(text))
    }

    /// Reads the string as `&str` without copying it.
    ///
    /// # Safety
    ///
    /// Differs from `borrow_as_str` in that this method returns does not check
    /// the string is a valid UTF-8 string.
    pub unsafe fn borrow_as_str_unchecked<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;
        let size = length.0 as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8_unchecked(&bytes[..size]);

        Ok(f(text))
    }
}

impl<'info, 'a> ZCMut<'info, 'a, FnkString<'info>> {
    // GETTERS ----------------------------------------------------------------

    pub fn len(&self) -> FankorResult<usize> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;

        Ok(length.0 as usize)
    }

    pub fn is_empty(&self) -> FankorResult<bool> {
        Ok(self.len()? == 0)
    }

    // METHODS ----------------------------------------------------------------

    /// Reads the string as `&str` without copying it.
    pub fn borrow_as_str<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;
        let size = length.0 as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8(&bytes[..size]).map_err(|_| {
            FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
        })?;

        Ok(f(text))
    }

    /// Reads the string as `&str` without copying it.
    ///
    /// # Safety
    ///
    /// Differs from `borrow_as_str` in that this method returns does not check
    /// the string is a valid UTF-8 string.
    pub unsafe fn borrow_as_str_unchecked<R, F: FnOnce(&str) -> R>(&self, f: F) -> FankorResult<R> {
        let bytes = self.info.data.borrow();
        let mut bytes = &bytes[self.offset..];
        let length = FnkUInt::deserialize(&mut bytes)?;
        let size = length.0 as usize;

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let text = std::str::from_utf8_unchecked(&bytes[..size]);

        Ok(f(text))
    }
}
