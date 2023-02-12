use crate::errors::FankorResult;
use solana_program::account_info::AccountInfo;

pub trait ZeroCopyType<'info>: Sized {
    // CONSTRUCTORS -----------------------------------------------------------

    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)>;

    // STATIC METHODS ---------------------------------------------------------

    /// Returns the size of the type in bytes.
    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize>;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait CopyType<'info>: Sized {
    type ZeroCopyType: ZeroCopyType<'info>;

    // METHODS ----------------------------------------------------------------

    /// Returns the size of the type in bytes from an instance.
    fn byte_size(&self) -> usize {
        Self::min_byte_size()
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Returns the minimum byte size of the type in bytes.
    fn min_byte_size() -> usize;
}
