use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{CopyType, ZeroCopyType};
use solana_program::account_info::AccountInfo;
use solana_program::program_option::COption;
use std::any::type_name;
use std::mem::size_of;

impl<'info, T: ZeroCopyType<'info>> ZeroCopyType<'info> for Option<T> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let flag = {
            let bytes =
                info.try_borrow_data()
                    .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: type_name::<Self>(),
                    })?;
            let bytes = &bytes[offset..];

            if bytes.is_empty() {
                return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                    type_name: type_name::<Self>(),
                }
                .into());
            }

            bytes[0] != 0
        };

        let result = if flag {
            let (result, size) = T::new(info, offset + 1)?;
            (Some(result), size.map(|size| size + 1))
        } else {
            (None, Some(1))
        };

        Ok(result)
    }

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let mut size = 1;
        let flag = bytes[0];

        if flag != 0 {
            size += T::read_byte_size_from_bytes(&bytes[1..])?;
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info>> CopyType<'info> for Option<T> {
    type ZeroCopyType = Option<T::ZeroCopyType>;

    fn byte_size_from_instance(&self) -> usize {
        match self {
            None => 1,
            Some(v) => 1 + v.byte_size_from_instance(),
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info, T: ZeroCopyType<'info>> ZeroCopyType<'info> for COption<T> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let size = size_of::<u32>();
        let flag = {
            let bytes =
                info.try_borrow_data()
                    .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: type_name::<Self>(),
                    })?;
            let bytes = &bytes[offset..];

            if bytes.len() < size {
                return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                    type_name: type_name::<Self>(),
                }
                .into());
            }

            let flag = u32::from_le_bytes(bytes[..size].try_into().unwrap());
            flag != 0
        };

        let result = if flag {
            let (result, s) = T::new(info, offset + size)?;
            (COption::Some(result), s.map(|s| s + size))
        } else {
            (COption::None, Some(size))
        };

        Ok(result)
    }

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = size_of::<u32>();

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        let flag = u32::from_le_bytes(bytes[..size].try_into().unwrap());

        if flag != 0 {
            size += T::read_byte_size_from_bytes(&bytes[size..])?;
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info>> CopyType<'info> for COption<T> {
    type ZeroCopyType = Option<T::ZeroCopyType>;

    fn byte_size_from_instance(&self) -> usize {
        match self {
            COption::None => size_of::<u32>(),
            COption::Some(v) => size_of::<u32>() + v.byte_size_from_instance(),
        }
    }
}
