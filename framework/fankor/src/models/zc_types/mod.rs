use crate::errors::{FankorErrorCode, FankorResult};
use borsh::{BorshDeserialize, BorshSerialize};
use std::any::type_name;
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

pub mod arrays;
pub mod maps;
pub mod numbers;
pub mod sets;
pub mod strings;
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

pub struct ZC<'info, T> {
    pub(crate) data: Rc<RefCell<&'info mut [u8]>>,
    pub(crate) offset: usize,
    pub(crate) _phantom: std::marker::PhantomData<T>,
}

impl<'info, T: ZeroCopyType> ZC<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new ZC from a slice of bytes.
    ///
    /// # Safety
    /// This method is unsafe because it does not check the offset.
    pub unsafe fn new(data: Rc<RefCell<&'info mut [u8]>>, offset: usize) -> Self {
        Self {
            data,
            offset,
            _phantom: std::marker::PhantomData,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the size of the type in bytes.
    /// Note: validates the type without deserializing it.
    pub fn byte_size(&self) -> FankorResult<usize> {
        let bytes = (*self.data).borrow();
        let bytes = &bytes[self.offset..];
        T::byte_size(bytes)
    }
}

impl<'info, T: ZeroCopyType + BorshDeserialize> ZC<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Gets the actual value of the type.
    ///
    /// # Safety
    ///
    /// This method can fail if `bytes` cannot be deserialized into the type.
    pub fn try_get_value(&self) -> FankorResult<T> {
        let bytes = (*self.data).borrow();
        let mut bytes = &bytes[self.offset..];
        Ok(T::deserialize(&mut bytes)?)
    }
}

impl<'info, T: ZeroCopyType + BorshSerialize> ZC<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Writes a value in the buffer.
    ///
    /// # Safety
    ///
    /// This method can fail if `value` does not fit in the space allocated for it.
    pub fn try_write_value(&mut self, value: &T) -> FankorResult<()> {
        let mut bytes = (*self.data).borrow_mut();
        let previous_size = T::byte_size(&bytes)?;
        let new_size = value.byte_size_from_instance();

        // Check to prevent overwriting next element.
        if new_size > previous_size {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<T>(),
            }
            .into());
        }

        let bytes = &mut bytes[self.offset..];
        let mut cursor = Cursor::new(bytes);
        T::serialize(value, &mut cursor)?;

        // Shrink the buffer
        if new_size < previous_size {
            let bytes = cursor.into_inner();
            bytes[new_size..].rotate_left(previous_size - new_size);
            bytes[new_size..previous_size].fill(0);
        }

        Ok(())
    }
}

impl<'info, T> Clone for ZC<'info, T> {
    fn clone(&self) -> Self {
        ZC {
            data: self.data.clone(),
            offset: self.offset,
            _phantom: std::marker::PhantomData,
        }
    }
}
