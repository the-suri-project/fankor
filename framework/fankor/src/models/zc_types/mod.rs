pub mod arrays;
pub mod bool;
pub mod boxed;
pub mod maps;
pub mod numbers;
pub mod options;
pub mod pubkeys;
pub mod ranges;
pub mod sets;
pub mod strings;
pub mod tuples;
pub mod vec;

use crate::errors::FankorResult;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use std::cmp::Ordering;
use std::io::Cursor;

pub trait ZeroCopyType<'info>: Sized {
    // CONSTRUCTORS -----------------------------------------------------------

    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)>;

    // STATIC METHODS ---------------------------------------------------------

    /// Returns the size of the type in bytes.
    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize>;
}

pub trait CopyType<'info>: Sized {
    type ZeroCopyType: ZeroCopyType<'info>;

    // METHODS ----------------------------------------------------------------

    /// Returns the size of the type in bytes from an instance.
    fn byte_size_from_instance(&self) -> usize;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A wrapper around a `T` that implements `ZeroCopyType`.
pub struct Zc<'info, T: CopyType<'info>> {
    pub(crate) info: &'info AccountInfo<'info>,
    pub(crate) offset: usize,
    pub(crate) _data: std::marker::PhantomData<T>,
}

impl<'info, T: CopyType<'info>> Zc<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new ZC from a slice of bytes.
    ///
    /// # Safety
    /// This method is unsafe because it does not check the offset.
    pub unsafe fn new(info: &'info AccountInfo<'info>, offset: usize) -> Self {
        Self {
            info,
            offset,
            _data: std::marker::PhantomData,
        }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn info(&self) -> &'info AccountInfo<'info> {
        self.info
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the size of the type in bytes.
    /// Note: validates the type without deserializing it.
    pub fn byte_size(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];
        T::ZeroCopyType::read_byte_size_from_bytes(bytes)
    }
}

impl<'info, T: CopyType<'info> + BorshDeserialize> Zc<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Gets the actual value of the type.
    ///
    /// # Safety
    ///
    /// This method can fail if `bytes` cannot be deserialized into the type.
    pub fn try_get_value(&self) -> FankorResult<T> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        Ok(T::deserialize(&mut bytes)?)
    }

    /// Gets the zero-copy version of the type.
    ///
    /// # Safety
    ///
    /// This method can fail if `bytes` cannot be deserialized into the type.
    pub fn get_zero_copy_value(&self) -> FankorResult<T::ZeroCopyType> {
        T::ZeroCopyType::new(self.info, self.offset).map(|(v, _)| v)
    }
}

impl<'info, T: CopyType<'info> + BorshSerialize> Zc<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Writes a value in the buffer occupying at most the same space as the
    /// previous value.
    ///
    /// # Safety
    ///
    /// This method can fail if `value` does not fit in the buffer.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub unsafe fn try_write_value(&self, value: &T) -> FankorResult<()> {
        let bytes = (*self.info.data).borrow();
        let previous_size = T::ZeroCopyType::read_byte_size_from_bytes(&bytes)?;
        let new_size = value.byte_size_from_instance();

        drop(bytes);

        self.try_write_value_with_sizes(value, previous_size, new_size)
    }

    /// Writes a value in the buffer occupying at most the same space as the
    /// previous value.
    ///
    /// # Safety
    ///
    /// This method can fail if `value` does not fit in the buffer or if any
    /// of the sizes are incorrect.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub unsafe fn try_write_value_with_sizes(
        &self,
        value: &T,
        previous_size: usize,
        new_size: usize,
    ) -> FankorResult<()> {
        let mut original_bytes = (*self.info.data).borrow_mut();

        match new_size.cmp(&previous_size) {
            Ordering::Less => {
                // Serialize
                let bytes = &mut original_bytes[self.offset..];
                let mut cursor = Cursor::new(bytes);
                value.serialize(&mut cursor)?;

                // Shift bytes
                let diff = previous_size - new_size;
                let bytes = cursor.into_inner();
                bytes[new_size..].rotate_left(diff);

                // Reallocate the buffer
                self.info.realloc(original_bytes.len() - diff, false)?;
            }
            Ordering::Equal => {
                // Serialize
                let bytes = &mut original_bytes[self.offset..];
                let mut cursor = Cursor::new(bytes);
                value.serialize(&mut cursor)?;
            }
            Ordering::Greater => {
                // Reallocate the buffer
                let diff = new_size - previous_size;
                self.info.realloc(original_bytes.len() + diff, false)?;

                // Shift bytes
                original_bytes[self.offset + previous_size..].rotate_right(diff);

                // Serialize
                let bytes = &mut original_bytes[self.offset..];
                let mut cursor = Cursor::new(bytes);
                value.serialize(&mut cursor)?;
            }
        }

        Ok(())
    }
}

impl<'info, T: CopyType<'info>> Clone for Zc<'info, T> {
    fn clone(&self) -> Self {
        Zc {
            info: self.info,
            offset: self.offset,
            _data: std::marker::PhantomData,
        }
    }
}
