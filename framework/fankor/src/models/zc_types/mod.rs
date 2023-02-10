pub mod arrays;
pub mod binary_vectors;
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

use crate::errors::{FankorErrorCode, FankorResult};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use std::cmp::Ordering;
use std::io::{Cursor, Read, Write};

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
    pub fn new_unchecked(info: &'info AccountInfo<'info>, offset: usize) -> Self {
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
        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let bytes = &bytes[self.offset..];
        T::ZeroCopyType::read_byte_size_from_bytes(bytes)
    }

    /// Reverses `length` bytes from the current offset expading the buffer and moving
    /// the rest bytes forward.
    ///
    /// # Safety
    ///
    /// This method can fail if there is not enough bytes to add.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn make_space(&self, length: usize) -> FankorResult<()> {
        if length == 0 {
            return Ok(());
        }

        // Reallocate the buffer
        let original_len = self.info.data_len();
        self.info.realloc(original_len + length, false)?;

        // Shift bytes
        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;
        let bytes = &mut bytes[self.offset..];
        bytes.copy_within(0..original_len - self.offset, length);

        Ok(())
    }

    /// Removes the data from the bytes.
    ///
    /// # Safety
    /// This method can fail if `value` was not present at the position.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn remove_unchecked(self) -> FankorResult<()> {
        let mut original_bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut original_bytes[self.offset..];
        let value_size = T::ZeroCopyType::read_byte_size_from_bytes(bytes)?;

        drop(original_bytes);

        self.remove_bytes_unchecked(value_size)?;

        Ok(())
    }

    /// Removes `length` bytes from the current offset.
    ///
    /// # Safety
    ///
    /// This method can fail if there is not enough bytes to remove.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn remove_bytes_unchecked(&self, length: usize) -> FankorResult<()> {
        if length == 0 {
            return Ok(());
        }

        let mut original_bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        // Shift bytes
        let bytes = &mut original_bytes[self.offset..];
        bytes.copy_within(length.., 0);

        // Reallocate the buffer
        let original_length = original_bytes.len();
        drop(original_bytes);

        self.info.realloc(original_length - length, false)?;

        Ok(())
    }

    /// Writes a byte slice in the buffer.
    ///
    /// # Safety
    /// This method can fail if `bytes` does not fit in the buffer or if any
    /// of the sizes are incorrect.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn try_write_bytes(&self, bytes: &[u8]) -> FankorResult<()> {
        let previous_size = self.byte_size()?;
        self.try_write_bytes_with_sizes_unchecked(bytes, previous_size)
    }

    /// Writes a byte slice in the buffer specifying the previous size.
    ///
    /// # Safety
    /// This method can fail if `bytes` does not fit in the buffer or if any
    /// of the sizes are incorrect.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn try_write_bytes_with_sizes_unchecked(
        &self,
        bytes: &[u8],
        previous_size: usize,
    ) -> FankorResult<()> {
        let original_len = self.info.data_len();
        let new_size = bytes.len();

        match new_size.cmp(&previous_size) {
            Ordering::Less => {
                // Serialize
                let mut original_bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let original_bytes_slice = &mut original_bytes[self.offset..];
                let mut cursor = Cursor::new(original_bytes_slice);
                cursor.write_all(bytes)?;

                // Shift bytes
                let diff = previous_size - new_size;
                let bytes = cursor.into_inner();
                bytes[new_size..].copy_within(diff.., 0);

                // Reallocate the buffer
                drop(original_bytes);

                self.info.realloc(original_len - diff, false)?;
            }
            Ordering::Equal => {
                // Serialize
                let mut original_bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let original_bytes_slice = &mut original_bytes[self.offset..];
                let mut cursor = Cursor::new(original_bytes_slice);
                cursor.write_all(bytes)?;
            }
            Ordering::Greater => {
                // Reallocate the buffer
                let diff = new_size - previous_size;
                self.info.realloc(original_len + diff, false)?;

                // Shift bytes
                let mut original_bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let original_bytes_slice = &mut original_bytes[self.offset..];
                original_bytes_slice
                    .copy_within(previous_size..original_len - self.offset, new_size);

                // Serialize
                let mut cursor = Cursor::new(original_bytes_slice);
                cursor.write_all(bytes)?;
            }
        }

        Ok(())
    }

    /// Moves a byte slice to a new position inside the buffer.
    ///
    /// # Safety
    /// This method can fail if `size` is incorrect.
    pub fn move_byte_slice(&self, from: usize, to: usize, size: usize) -> FankorResult<()> {
        if size == 0 {
            return Ok(());
        }

        match from.cmp(&to) {
            Ordering::Less => {
                let end = from + size;
                if end > to {
                    return Err(FankorErrorCode::ZeroCopyInvalidMove.into());
                }

                let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let bytes = &mut bytes[self.offset..];
                bytes[from..to].rotate_left(size);
            }
            Ordering::Equal => {
                // Same position
                return Ok(());
            }
            Ordering::Greater => {
                let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let bytes = &mut bytes[self.offset..];
                let end = from + size;
                bytes[to..end].rotate_right(size);
            }
        }

        Ok(())
    }
}

impl<'info, T: CopyType<'info> + BorshDeserialize> Zc<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Gets the actual value of the type.
    ///
    /// # Safety
    ///
    /// This method can fail if `bytes` cannot be deserialized into the type.
    pub fn try_value(&self) -> FankorResult<T> {
        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &bytes[self.offset..];
        Ok(T::deserialize(&mut bytes)?)
    }

    /// Gets the zero-copy version of the type.
    ///
    /// # Safety
    ///
    /// This method can fail if `bytes` cannot be deserialized into the type.
    pub fn zc_value(&self) -> FankorResult<T::ZeroCopyType> {
        T::ZeroCopyType::new(self.info, self.offset).map(|(v, _)| v)
    }
}

impl<'info, T: CopyType<'info> + BorshSerialize> Zc<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Writes a value in the buffer.
    ///
    /// # Safety
    /// This method can fail if `value` does not fit in the buffer.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn try_write_value_unchecked(&self, value: &T) -> FankorResult<()> {
        let original_bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let bytes = &original_bytes[self.offset..];
        let previous_size = T::ZeroCopyType::read_byte_size_from_bytes(bytes)?;
        let new_size = value.byte_size_from_instance();

        drop(original_bytes);

        self.try_write_value_with_sizes_unchecked(value, previous_size, new_size)
    }

    /// Writes a value in the buffer specifying the previous and new sizes.
    ///
    /// # Safety
    /// This method can fail if `value` does not fit in the buffer or if any
    /// of the sizes are incorrect.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn try_write_value_with_sizes_unchecked(
        &self,
        value: &T,
        previous_size: usize,
        new_size: usize,
    ) -> FankorResult<()> {
        let original_len = self.info.data_len();

        match new_size.cmp(&previous_size) {
            Ordering::Less => {
                // Serialize
                let mut original_bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let bytes = &mut original_bytes[self.offset..];
                let mut cursor = Cursor::new(bytes);
                value.serialize(&mut cursor)?;

                // Shift bytes
                let diff = previous_size - new_size;
                let bytes = cursor.into_inner();
                bytes[new_size..].copy_within(diff.., 0);

                // Reallocate the buffer
                drop(original_bytes);

                self.info.realloc(original_len - diff, false)?;
            }
            Ordering::Equal => {
                // Serialize
                let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let bytes = &mut bytes[self.offset..];
                let mut cursor = Cursor::new(bytes);
                value.serialize(&mut cursor)?;
            }
            Ordering::Greater => {
                // Reallocate the buffer
                let diff = new_size - previous_size;
                self.info.realloc(original_len + diff, false)?;

                // Shift bytes
                let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<Self>(),
                    }
                })?;
                let bytes = &mut bytes[self.offset..];
                bytes.copy_within(previous_size..original_len - self.offset, new_size);

                // Serialize
                let mut cursor = Cursor::new(bytes);
                value.serialize(&mut cursor)?;
            }
        }

        Ok(())
    }
}

impl<'info, T: CopyType<'info>> Zc<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Generates a new ZC at the given `position` starting from the current offset.
    ///
    /// # Safety
    /// This method can fail if `position` is out of bounds.
    pub fn zc_at_unchecked<V: CopyType<'info>>(
        &self,
        position: usize,
    ) -> FankorResult<Zc<'info, V>> {
        Ok(Zc::new_unchecked(self.info, self.offset + position))
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

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    pub fn test_move_byte_slice() {
        // Positive cases
        for (from, to, size, result) in [
            (3, 3, 3, vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            (3, 6, 3, vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            (3, 7, 3, vec![0u8, 1, 2, 6, 3, 4, 5, 7, 8, 9, 10, 11, 12]),
            (3, 13, 3, vec![0u8, 1, 2, 6, 7, 8, 9, 10, 11, 12, 3, 4, 5]),
            (7, 3, 3, vec![0u8, 1, 2, 7, 8, 9, 3, 4, 5, 6, 10, 11, 12]),
            (7, 6, 3, vec![0u8, 1, 2, 3, 4, 5, 7, 8, 9, 6, 10, 11, 12]),
            (7, 0, 3, vec![7, 8, 9, 0u8, 1, 2, 3, 4, 5, 6, 10, 11, 12]),
            (0, 13, 13, vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            (13, 12, 0, vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        ] {
            let mut lamports = 0;
            let mut data = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
            let info = AccountInfo {
                key: &Default::default(),
                is_signer: false,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut lamports)),
                data: Rc::new(RefCell::new(&mut data)),
                owner: &Default::default(),
                executable: false,
                rent_epoch: 0,
            };

            let zc = Zc::<u8>::new_unchecked(&info, 0);
            assert!(
                zc.move_byte_slice(from, to, size).is_ok(),
                "Failed to move bytes"
            );
            assert_eq!(*info.try_borrow_data().unwrap(), result);
        }

        // Negative cases
        for (from, to, size) in [(3, 4, 3), (3, 5, 3), (0, 5, 9), (0, 6, 9), (0, 10, 14)] {
            let mut lamports = 0;
            let mut data = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
            let info = AccountInfo {
                key: &Default::default(),
                is_signer: false,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut lamports)),
                data: Rc::new(RefCell::new(&mut data)),
                owner: &Default::default(),
                executable: false,
                rent_epoch: 0,
            };

            let zc = Zc::<u8>::new_unchecked(&info, 0);
            assert!(
                zc.move_byte_slice(from, to, size).is_err(),
                "Move bytes must fail for ({},{},{})",
                from,
                to,
                size
            );
        }
    }
}
