use std::cmp::Ordering;
use std::io::{Cursor, Write};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::traits::{CopyType, ZeroCopyType};

pub mod arrays;
pub mod binary_map;
pub mod binary_set;
pub mod bool;
pub mod boxed;
pub mod extensions;
pub mod numbers;
pub mod options;
pub mod pubkeys;
pub mod ranges;
pub mod strings;
pub mod tuples;
pub mod vec;

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
        T::ZeroCopyType::read_byte_size(bytes)
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
        #[cfg(any(feature = "test-utils", test))]
        if self.info.rent_epoch == crate::tests::ACCOUNT_INFO_TEST_MAGIC_NUMBER {
            let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
                FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                }
            })?;
            let bytes = &mut bytes[self.offset..];
            bytes.rotate_right(length);
            return Ok(());
        }

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
        let value_size = T::ZeroCopyType::read_byte_size(bytes)?;

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

        #[cfg(any(feature = "test-utils", test))]
        if self.info.rent_epoch == crate::tests::ACCOUNT_INFO_TEST_MAGIC_NUMBER {
            self.info.realloc(original_length - length, false)?;
        }

        #[cfg(not(any(feature = "test-utils", test)))]
        {
            self.info.realloc(original_length - length, false)?;
        }

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

                #[cfg(any(feature = "test-utils", test))]
                if self.info.rent_epoch == crate::tests::ACCOUNT_INFO_TEST_MAGIC_NUMBER {
                    self.info.realloc(original_len - diff, false)?;
                }

                #[cfg(not(any(feature = "test-utils", test)))]
                {
                    self.info.realloc(original_len - diff, false)?;
                }
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

                #[cfg(any(feature = "test-utils", test))]
                if self.info.rent_epoch == crate::tests::ACCOUNT_INFO_TEST_MAGIC_NUMBER {
                    // Shift bytes
                    let mut original_bytes = self.info.data.try_borrow_mut().map_err(|_| {
                        FankorErrorCode::ZeroCopyPossibleDeadlock {
                            type_name: std::any::type_name::<Self>(),
                        }
                    })?;
                    let original_bytes_slice = &mut original_bytes[self.offset..];
                    original_bytes_slice.rotate_right(diff);

                    // Serialize
                    let mut cursor = Cursor::new(original_bytes_slice);
                    cursor.write_all(bytes)?;

                    return Ok(());
                }

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

    /// Appends the bytes of this element to a vector.
    /// This method is used to map the bytes of one element to another.
    ///
    /// # Safety
    /// This method can fail if the element does not fit in `buffer`.
    pub fn append_to_vec(&self, buffer: &mut Vec<u8>) -> FankorResult<()> {
        let size = {
            let original_bytes = self.info.data.try_borrow().map_err(|_| {
                FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                }
            })?;
            let original_bytes_slice = &original_bytes[self.offset..];
            T::ZeroCopyType::read_byte_size(original_bytes_slice)?
        };

        self.append_to_vec_with_size(buffer, size)
    }

    /// Appends the bytes of this element to a vector.
    /// This method is used to map the bytes of one element to another.
    ///
    /// # Safety
    /// This method can fail if the element does not fit in `buffer`.
    pub fn append_to_vec_with_size(&self, buffer: &mut Vec<u8>, size: usize) -> FankorResult<()> {
        let original_bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &original_bytes[self.offset..];
        bytes = &bytes[..size];

        buffer.extend_from_slice(bytes);

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
        let previous_size = T::ZeroCopyType::read_byte_size(bytes)?;
        let new_size = value.byte_size();

        drop(original_bytes);

        self.try_write_value_with_sizes_unchecked(value, previous_size, new_size)
    }

    /// Writes a zero-copy value in the buffer.
    ///
    /// # Safety
    /// This method can fail if `value` does not fit in the buffer.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn try_write_zc_value_unchecked(&self, value: &Zc<'info, T>) -> FankorResult<()> {
        let original_bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let bytes = &original_bytes[self.offset..];
        let previous_size = T::ZeroCopyType::read_byte_size(bytes)?;

        drop(original_bytes);

        let original_value_bytes = value.info.data.try_borrow().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;
        let value_bytes = &original_value_bytes[self.offset..];
        let value_size = T::ZeroCopyType::read_byte_size(value_bytes)?;
        let value_bytes = &value_bytes[..value_size];

        self.try_write_bytes_with_sizes_unchecked(value_bytes, previous_size)
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

                #[cfg(any(feature = "test-utils", test))]
                if self.info.rent_epoch == crate::tests::ACCOUNT_INFO_TEST_MAGIC_NUMBER {
                    self.info.realloc(original_len - diff, false)?;
                }

                #[cfg(not(any(feature = "test-utils", test)))]
                {
                    self.info.realloc(original_len - diff, false)?;
                }
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

                #[cfg(any(feature = "test-utils", test))]
                if self.info.rent_epoch == crate::tests::ACCOUNT_INFO_TEST_MAGIC_NUMBER {
                    // Shift bytes
                    let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
                        FankorErrorCode::ZeroCopyPossibleDeadlock {
                            type_name: std::any::type_name::<Self>(),
                        }
                    })?;
                    let bytes = &mut bytes[self.offset..];
                    bytes.rotate_right(diff);

                    // Serialize
                    let mut cursor = Cursor::new(bytes);
                    value.serialize(&mut cursor)?;

                    return Ok(());
                }

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

    /// Writes a zero-copy value in the buffer specifying the previous size.
    ///
    /// # Safety
    /// This method can fail if `value` does not fit in the buffer.
    ///
    /// MAKE SURE THAT THIS IS THE ONLY REFERENCE TO THE SAME ACCOUNT, OTHERWISE
    /// YOU WILL OVERWRITE DATA.
    pub fn try_write_zc_value_with_size_unchecked(
        &self,
        value: &Zc<'info, T>,
        previous_size: usize,
    ) -> FankorResult<()> {
        let original_value_bytes = value.info.data.try_borrow().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;
        let value_bytes = &original_value_bytes[self.offset..];
        let value_size = T::ZeroCopyType::read_byte_size(value_bytes)?;
        let value_bytes = &value_bytes[..value_size];

        self.try_write_bytes_with_sizes_unchecked(value_bytes, previous_size)
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
    use crate::tests::create_account_info_for_tests;

    use super::*;

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
            let info = create_account_info_for_tests(&mut lamports, &mut data);
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
            let info = create_account_info_for_tests(&mut lamports, &mut data);
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
