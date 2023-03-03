use crate::errors::{FankorErrorCode, FankorResult};
use crate::traits::{CopyType, ZeroCopyType};
use solana_program::account_info::AccountInfo;

impl<'info> ZeroCopyType<'info> for bool {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes = info
            .try_borrow_data()
            .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock { type_name: "bool" })?;
        let bytes = &bytes[offset..];

        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: "bool" }.into());
        }

        Ok((bytes[0] != 0, Some(1)))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: "bool" }.into());
        }

        Ok(1)
    }
}

impl<'info> CopyType<'info> for bool {
    type ZeroCopyType = bool;

    fn min_byte_size() -> usize {
        1
    }
}
