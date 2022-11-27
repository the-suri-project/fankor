use crate::prelude::{FnkRange, FnkURange};
use crate::{
    errors::FankorErrorCode,
    errors::FankorResult,
    models::CopyType,
    models::ZeroCopyType,
    prelude::{FnkInt, FnkUInt},
    traits::AccountSize,
};
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

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

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = FnkUInt::read_byte_size_from_bytes(bytes)?;
        size += FnkUInt::read_byte_size_from_bytes(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info> CopyType<'info> for FnkURange {
    type ZeroCopyType = FnkURange;

    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
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

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = FnkInt::read_byte_size_from_bytes(bytes)?;
        size += FnkInt::read_byte_size_from_bytes(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info> CopyType<'info> for FnkRange {
    type ZeroCopyType = FnkRange;

    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }
}
