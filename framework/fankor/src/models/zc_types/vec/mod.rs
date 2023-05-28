use std::marker::PhantomData;
use std::mem::size_of;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;

pub use fnk::*;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::Zc;
use crate::traits::{CopyType, ZeroCopyType};
use crate::utils::writers::ArrayWriter;

mod fnk;

pub struct ZcVec<'info, T: CopyType<'info>> {
    info: &'info AccountInfo<'info>,
    offset: usize,
    _data: PhantomData<T>,
}

impl<'info, T: CopyType<'info>> ZeroCopyType<'info> for ZcVec<'info, T> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((
            ZcVec {
                info,
                offset,
                _data: PhantomData,
            },
            None,
        ))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut bytes2 = bytes;
        let len = u32::deserialize(&mut bytes2)?;
        let mut size = size_of::<u32>();

        match size_of::<T>() {
            0 => {}
            1 => size += len as usize,
            _ => {
                for _ in 0..len {
                    size += T::ZeroCopyType::read_byte_size(&bytes[size..])?;
                }
            }
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info>> CopyType<'info> for Vec<T> {
    type ZeroCopyType = ZcVec<'info, T>;

    fn byte_size(&self) -> usize {
        size_of::<u32>() // Length
            + self.iter().map(|x| x.byte_size()).sum::<usize>()
    }

    fn min_byte_size() -> usize {
        size_of::<u32>() // Length
    }
}

impl<'info, T: CopyType<'info>> ZcVec<'info, T> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the vector.
    pub fn len(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let len = u32::deserialize(&mut bytes)?;

        Ok(len as usize)
    }

    /// Whether the vector is empty or not
    pub fn is_empty(&self) -> FankorResult<bool> {
        Ok(self.len()? == 0)
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the element at the specified position.
    pub fn get_zc_index(&self, index: usize) -> FankorResult<Option<Zc<'info, T>>> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let initial_size = bytes.len();

        let len = u32::deserialize(&mut bytes)?;

        let index = index as u32;
        if index >= len {
            return Ok(None);
        }

        for i in 0..len {
            if i == index {
                return Ok(Some(Zc {
                    info: self.info,
                    offset: self.offset + initial_size - bytes.len(),
                    _data: PhantomData,
                }));
            }

            let size = T::ZeroCopyType::read_byte_size(bytes)?;
            bytes = &bytes[size..];
        }

        Ok(None)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// # Safety
    ///
    /// DO NOT WRITE TO THE ACCOUNT WHILE INSIDE THE PREDICATE.
    pub fn retain<F>(&self, mut f: F) -> FankorResult<()>
    where
        F: FnMut(&Zc<T>) -> FankorResult<bool>,
    {
        let mut offset = self.offset + size_of::<u32>();
        let mut length = self.len()?;

        #[allow(clippy::mut_range_bound)]
        for _ in 0..length {
            let zc = Zc::<T>::new_unchecked(self.info, offset);

            if !f(&zc)? {
                zc.remove_unchecked()?;
                length -= 1;
            } else {
                offset += zc.byte_size()?;
            }
        }

        self.write_len_unchecked(length as u32)?;

        Ok(())
    }

    pub fn iter(&self) -> Iter<'info, T> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let original_len = bytes.len();
        let len = u32::deserialize(&mut bytes).expect("Failed to get length of ZcVec in iterator");

        Iter {
            info: self.info,
            offset: self.offset + (original_len - bytes.len()),
            len: len as usize,
            index: 0,
            _data: PhantomData,
        }
    }

    /// Writes the length of the vector.
    pub fn write_len_unchecked(&self, new_length: u32) -> FankorResult<()> {
        let mut bytes = (*self.info.data).borrow_mut();
        let bytes = &mut bytes[self.offset..];
        let mut writer = ArrayWriter::new(bytes);
        u32::serialize(&new_length, &mut writer)?;

        Ok(())
    }
}

impl<'info, T: CopyType<'info> + BorshSerialize> ZcVec<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Appends a list of elements to the end of the vector.
    /// Returns the size of the vector in bytes.
    pub fn append(&self, values: &[T]) -> FankorResult<usize> {
        // Get current size.
        let mut size = {
            let bytes = (*self.info.data).borrow();
            let bytes = &bytes[self.offset..];
            Self::read_byte_size(bytes)?
        };

        // Update length.
        let length = self.len()?;
        let new_length = length
            .checked_add(values.len())
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?;
        let new_length =
            u32::try_from(new_length).map_err(|_| FankorErrorCode::ZeroCopyLengthFieldOverflow)?;
        self.write_len_unchecked(new_length)?;

        // Append values.
        for value in values {
            let zc = Zc::new_unchecked(self.info, self.offset + size);
            let v_size = value.byte_size();
            zc.try_write_value_with_sizes_unchecked(value, 0, v_size)?;
            size += v_size;
        }

        Ok(size)
    }

    /// Appends a list of zero-copy elements to the end of the vector.
    /// Returns the size of the vector in bytes.
    pub fn append_zc(&self, values: &[Zc<'info, T>]) -> FankorResult<usize> {
        // Get current size.
        let mut size = {
            let bytes = (*self.info.data).borrow();
            let bytes = &bytes[self.offset..];
            Self::read_byte_size(bytes)?
        };

        // Update length.
        let length = self.len()?;
        let new_length = length
            .checked_add(values.len())
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?;
        let new_length =
            u32::try_from(new_length).map_err(|_| FankorErrorCode::ZeroCopyLengthFieldOverflow)?;
        self.write_len_unchecked(new_length)?;

        // Append values.
        for value in values {
            let original_value_bytes = value.info.data.try_borrow().map_err(|_| {
                FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                }
            })?;
            let value_bytes = &original_value_bytes[self.offset..];
            let value_size = T::ZeroCopyType::read_byte_size(value_bytes)?;
            let value_bytes = &value_bytes[..value_size];

            let zc = Zc::<T>::new_unchecked(self.info, self.offset + size);
            zc.try_write_bytes_with_sizes_unchecked(value_bytes, 0)?;
            size += value_size;
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info>> IntoIterator for ZcVec<'info, T> {
    type Item = Zc<'info, T>;
    type IntoIter = Iter<'info, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct Iter<'info, T: CopyType<'info>> {
    pub(crate) info: &'info AccountInfo<'info>,
    pub(crate) len: usize,
    pub(crate) index: usize,
    pub(crate) offset: usize,
    pub(crate) _data: PhantomData<T>,
}

impl<'info, T: CopyType<'info>> Iterator for Iter<'info, T> {
    type Item = Zc<'info, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        if self.index != 0 {
            let bytes = (*self.info.data).borrow();
            let bytes = &bytes[self.offset..];

            self.offset += T::ZeroCopyType::read_byte_size(bytes)
                .expect("Deserialization failed in vector iterator");
        }

        let result = Zc {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        };

        self.index += 1;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;

        (size, Some(size))
    }
}

impl<'info, T: CopyType<'info>> ExactSizeIterator for Iter<'info, T> {}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;

    use solana_program::pubkey::Pubkey;

    use crate::tests::create_account_info_for_tests;

    use super::*;

    #[test]
    fn test_read_byte_length() {
        let vector = vec![2, 0, 0, 0, 1, 0, 2, 0, 99];
        let size = ZcVec::<u16>::read_byte_size(&vector).unwrap();

        assert_eq!(size, size_of::<u32>() + 2 * size_of::<u16>());
    }

    #[test]
    fn test_len_and_iter() {
        let mut lamports = 0;
        let mut vector = vec![5, 0, 0, 0, 3, 3, 3, 3, 3];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcVec::<u8>::new(&info, 0).unwrap();

        assert_eq!(zc.len().unwrap(), 5);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, 3);
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn test_iter_modifying() {
        let mut lamports = 0;
        let mut vector = vec![5, 0, 0, 0, 1, 3, 1, 2, 1, 3, 1, 2, 1, 3];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcVec::<Option<u8>>::new(&info, 0).unwrap();

        zc.iter().for_each(|zc_el| {
            let value = zc_el.try_value().unwrap().unwrap();

            if value == 2 {
                zc_el.try_write_value_unchecked(&None).unwrap();
            }
        });

        let mut some_count = 0;
        let mut none_count = 0;
        for zc_el in zc {
            match zc_el.try_value().unwrap() {
                Some(v) => {
                    assert_eq!(v, 3);
                    some_count += 1;
                }
                None => {
                    none_count += 1;
                }
            }
        }

        assert_eq!(some_count, 3);
        assert_eq!(none_count, 2);
    }

    #[test]
    fn test_write_len() {
        let mut lamports = 0;
        let mut vector = vec![3, 0, 0, 0, 3, 3, 3, 3, 3];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcVec::<u8>::new(&info, 0).unwrap();
        zc.write_len_unchecked(5).unwrap();

        assert_eq!(zc.len().unwrap(), 5);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, 3);
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn test_append() {
        let mut lamports = 0;
        let mut vector = vec![3, 0, 0, 0, 3, 3, 3, 0, 0];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcVec::<u8>::new(&info, 0).unwrap();
        let new_offset = zc.append(&[3, 3]).unwrap();

        assert_eq!(zc.len().unwrap(), 5);
        assert_eq!(new_offset, 9);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, 3);
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn test_append_zc() {
        let mut lamports = 0;
        let mut vector = vec![3, 0, 0, 0, 3, 3, 3, 0, 0];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let mut lamports_el = 0;
        let mut vector_el = vec![3; 1];
        let info_el = AccountInfo {
            key: &Pubkey::default(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut lamports_el)),
            data: Rc::new(RefCell::new(&mut vector_el)),
            owner: &Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        };

        let (zc, _) = ZcVec::<u8>::new(&info, 0).unwrap();
        let zc_el = Zc::<u8>::new_unchecked(&info_el, 0);
        let new_offset = zc.append_zc(&[zc_el.clone(), zc_el]).unwrap();

        assert_eq!(zc.len().unwrap(), 5);
        assert_eq!(new_offset, 9);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, 3);
        }

        assert_eq!(count, 5);
    }
}
