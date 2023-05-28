use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

use crate::{
    errors::FankorErrorCode,
    errors::FankorResult,
    prelude::{FnkInt, FnkUInt},
};
use crate::prelude::{FnkRange, FnkURange};
use crate::traits::{CopyType, ZeroCopyType};

impl<'info> ZeroCopyType<'info> for FnkURange {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &bytes[offset..];
        let initial_size = bytes.len();
        let value = FnkURange::deserialize(&mut bytes)?;

        Ok((value, Some(initial_size - bytes.len())))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = FnkUInt::read_byte_size(bytes)?;
        size += FnkUInt::read_byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info> CopyType<'info> for FnkURange {
    type ZeroCopyType = FnkURange;

    fn byte_size(&self) -> usize {
        let (point, length) = self.point_and_length();

        point.byte_size() + length.byte_size()
    }

    fn min_byte_size() -> usize {
        FnkUInt::min_byte_size() + FnkInt::min_byte_size()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info> ZeroCopyType<'info> for FnkRange {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &bytes[offset..];
        let initial_size = bytes.len();
        let value = FnkRange::deserialize(&mut bytes)?;

        Ok((value, Some(initial_size - bytes.len())))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = FnkInt::read_byte_size(bytes)?;
        size += FnkInt::read_byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info> CopyType<'info> for FnkRange {
    type ZeroCopyType = FnkRange;

    fn byte_size(&self) -> usize {
        self.from().byte_size() + self.to().byte_size()
    }

    fn min_byte_size() -> usize {
        FnkInt::min_byte_size() * 2
    }
}
