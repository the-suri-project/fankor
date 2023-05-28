use solana_program::account_info::AccountInfo;

use crate::errors::FankorResult;
use crate::traits::{CopyType, ZeroCopyType};

impl<'info, T: ZeroCopyType<'info>> ZeroCopyType<'info> for Box<T> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let (result, size) = T::new(info, offset)?;
        Ok((Box::new(result), size))
    }

    #[inline(always)]
    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        T::read_byte_size(bytes)
    }
}

impl<'info, T: CopyType<'info>> CopyType<'info> for Box<T> {
    type ZeroCopyType = T::ZeroCopyType;

    fn byte_size(&self) -> usize {
        let aux: &T = self;
        aux.byte_size()
    }

    fn min_byte_size() -> usize {
        // Prevents infinite recursion.
        0
    }
}
