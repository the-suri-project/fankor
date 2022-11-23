use crate::errors::FankorResult;
use crate::models::{CopyType, ZeroCopyType};
use solana_program::account_info::AccountInfo;

impl<'info, T: ZeroCopyType<'info>> ZeroCopyType<'info> for Box<T> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let (result, size) = T::new(info, offset)?;
        Ok((Box::new(result), size))
    }

    #[inline(always)]
    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        T::read_byte_size_from_bytes(bytes)
    }
}

impl<'info, T: CopyType<'info>> CopyType<'info> for Box<T> {
    type ZeroCopyType = Option<T::ZeroCopyType>;

    fn byte_size_from_instance(&self) -> usize {
        let aux: &T = self;
        aux.byte_size_from_instance()
    }
}
