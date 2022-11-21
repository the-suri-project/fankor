use crate::errors::FankorResult;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use std::cmp::Ordering;
use std::io::Cursor;

pub mod arrays;
// pub mod maps;
pub mod bool;
pub mod numbers;
pub mod options;
pub mod pubkey;
pub mod sets;
pub mod strings;
pub mod tuples;
pub mod vectors;

pub trait ZeroCopyType: Sized {
    /// Returns the size of the type in bytes from an instance.
    fn byte_size_from_instance(&self) -> usize;

    /// Returns the size of the type in bytes.
    fn byte_size(bytes: &[u8]) -> FankorResult<usize>;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A readonly wrapper around a `T` that implements `ZeroCopyType`.
pub struct ZC<'info, 'a, T: ZeroCopyType> {
    pub(crate) info: &'info AccountInfo<'info>,
    pub(crate) offset: usize,
    pub(crate) _data: std::marker::PhantomData<(T, &'a ())>,
}

impl<'info, 'a, T: ZeroCopyType> ZC<'info, 'a, T> {
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

    /// Returns the size of the type in bytes.
    /// Note: validates the type without deserializing it.
    pub fn byte_size(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];
        T::byte_size(bytes)
    }
}

impl<'info, 'a, T: ZeroCopyType + BorshDeserialize> ZC<'info, 'a, T> {
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
}

impl<'info, 'a, T: ZeroCopyType> Clone for ZC<'info, 'a, T> {
    fn clone(&self) -> Self {
        ZC {
            info: self.info,
            offset: self.offset,
            _data: std::marker::PhantomData,
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A mutable wrapper around a `T` that implements `ZeroCopyType`.
pub struct ZCMut<'info, 'a, T: ZeroCopyType> {
    pub(crate) info: &'info AccountInfo<'info>,
    pub(crate) offset: usize,
    pub(crate) _data: std::marker::PhantomData<(T, &'a mut ())>,
}

impl<'info, 'a, T: ZeroCopyType> ZCMut<'info, 'a, T> {
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

    /// Returns the size of the type in bytes.
    /// Note: validates the type without deserializing it.
    pub fn byte_size(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];
        T::byte_size(bytes)
    }

    /// Returns the reference to the data as a readonly reference.
    pub fn to_ref(&self) -> ZC<'info, 'a, T> {
        ZC {
            info: self.info,
            offset: self.offset,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType + BorshDeserialize> ZCMut<'info, 'a, T> {
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
}

impl<'info, 'a, T: ZeroCopyType + BorshSerialize> ZCMut<'info, 'a, T> {
    // METHODS ----------------------------------------------------------------

    /// Writes a value in the buffer occupying at most the same space as the
    /// previous value.
    ///
    /// # Safety
    ///
    /// This method can fail if `value` does not fit in the buffer.
    pub fn try_write_value(&mut self, value: &T) -> FankorResult<()> {
        let bytes = (*self.info.data).borrow();
        let previous_size = T::byte_size(&bytes)?;
        let new_size = value.byte_size_from_instance();

        drop(bytes);

        unsafe { self.try_write_value_with_sizes(value, previous_size, new_size) }
    }

    /// Writes a value in the buffer occupying at most the same space as the
    /// previous value.
    ///
    /// # Safety
    ///
    /// This method can fail if `value` does not fit in the buffer or if any
    /// of the sizes are incorrect.
    pub unsafe fn try_write_value_with_sizes(
        &mut self,
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
                T::serialize(value, &mut cursor)?;

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
                T::serialize(value, &mut cursor)?;
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
                T::serialize(value, &mut cursor)?;
            }
        }

        Ok(())
    }
}
