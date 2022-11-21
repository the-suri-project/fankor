use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{ZCMut, ZeroCopyType, ZC};
use solana_program::program_option::COption;
use std::any::type_name;
use std::marker::PhantomData;
use std::mem::size_of;

impl<T: ZeroCopyType> ZeroCopyType for Option<T> {
    fn byte_size_from_instance(&self) -> usize {
        match self {
            None => 1,
            Some(v) => 1 + v.byte_size_from_instance(),
        }
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let mut size = 1;
        let flag = bytes[0];

        if flag != 0 {
            bytes = &bytes[1..];
            size += T::byte_size(bytes)?;
        }

        Ok(size)
    }
}

impl<'info, 'a, T: ZeroCopyType> ZC<'info, 'a, Option<T>> {
    // GETTERS ----------------------------------------------------------------

    pub fn is_some(&self) -> FankorResult<bool> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = bytes[0];
        Ok(flag != 0)
    }

    pub fn is_none(&self) -> FankorResult<bool> {
        Ok(!self.is_some()?)
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the ZC of the inner value as a regular option.
    pub fn to_option(&self) -> FankorResult<Option<ZC<'info, 'a, T>>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = bytes[0];

        if flag == 0 {
            Ok(None)
        } else {
            Ok(Some(ZC {
                info: self.info,
                offset: self.offset + 1,
                _data: PhantomData,
            }))
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> ZCMut<'info, 'a, Option<T>> {
    // METHODS ----------------------------------------------------------------

    /// Gets the ZCMut of the inner value as a regular option.
    pub fn to_option_mut(&mut self) -> FankorResult<Option<ZCMut<'info, 'a, T>>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = bytes[0];

        if flag == 0 {
            Ok(None)
        } else {
            Ok(Some(ZCMut {
                info: self.info,
                offset: self.offset + 1,
                _data: PhantomData,
            }))
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<T: ZeroCopyType> ZeroCopyType for COption<T> {
    fn byte_size_from_instance(&self) -> usize {
        match self {
            COption::None => size_of::<u32>(),
            COption::Some(v) => size_of::<u32>() + v.byte_size_from_instance(),
        }
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = size_of::<u32>();
        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = u32::from_le_bytes(bytes[..size].try_into().unwrap());

        if flag != 0 {
            size += T::byte_size(&bytes[size..])?;
        }

        Ok(size)
    }
}

impl<'info, 'a, T: ZeroCopyType> ZC<'info, 'a, COption<T>> {
    // GETTERS ----------------------------------------------------------------

    pub fn is_some(&self) -> FankorResult<bool> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        let size = size_of::<u32>();
        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = u32::from_le_bytes(bytes[..size].try_into().unwrap());
        Ok(flag != 0)
    }

    pub fn is_none(&self) -> FankorResult<bool> {
        Ok(!self.is_some()?)
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the ZC of the inner value as a regular option.
    pub fn to_option(&self) -> FankorResult<COption<ZC<'info, 'a, T>>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        let size = size_of::<u32>();
        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = u32::from_le_bytes(bytes[..size].try_into().unwrap());

        if flag == 0 {
            Ok(COption::None)
        } else {
            Ok(COption::Some(ZC {
                info: self.info,
                offset: self.offset + size,
                _data: PhantomData,
            }))
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> ZCMut<'info, 'a, COption<T>> {
    // METHODS ----------------------------------------------------------------

    /// Gets the ZCMut of the inner value as a regular option.
    pub fn to_option_mut(&mut self) -> FankorResult<COption<ZCMut<'info, 'a, T>>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        let size = size_of::<u32>();
        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = u32::from_le_bytes(bytes[..size].try_into().unwrap());

        if flag == 0 {
            Ok(COption::None)
        } else {
            Ok(COption::Some(ZCMut {
                info: self.info,
                offset: self.offset + size,
                _data: PhantomData,
            }))
        }
    }
}
