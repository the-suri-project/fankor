use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::zc_types::vec::Iter;
use crate::models::Zc;
use crate::prelude::{FnkMap, FnkSet, FnkUInt, FnkVec};
use crate::traits::{CopyType, ZeroCopyType};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use std::marker::PhantomData;
use std::mem::size_of;

pub struct ZcFnkVec<'info, T: CopyType<'info>> {
    info: &'info AccountInfo<'info>,
    offset: usize,
    _data: PhantomData<T>,
}

impl<'info, T: CopyType<'info>> ZeroCopyType<'info> for ZcFnkVec<'info, T> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((
            ZcFnkVec {
                info,
                offset,
                _data: PhantomData,
            },
            None,
        ))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut bytes2 = bytes;
        let len = FnkUInt::deserialize(&mut bytes2)?;
        let mut size = bytes.len() - bytes2.len();

        match size_of::<T>() {
            0 => {}
            1 => {
                size += len
                    .get_usize()
                    .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?
            }
            _ => {
                for _ in 0..len.0 {
                    size += T::ZeroCopyType::read_byte_size(&bytes[size..])?;
                }
            }
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info>> CopyType<'info> for FnkVec<T> {
    type ZeroCopyType = ZcFnkVec<'info, T>;

    fn byte_size(&self) -> usize {
        let mut size = 0;

        let len = FnkUInt::from(self.len() as u64);
        size += len.byte_size();

        for i in &self.0 {
            size += i.byte_size();
        }

        size
    }

    fn min_byte_size() -> usize {
        FnkUInt::min_byte_size()
    }
}

impl<'info, T: CopyType<'info> + Ord> CopyType<'info> for FnkSet<T> {
    type ZeroCopyType = ZcFnkVec<'info, T>;

    fn byte_size(&self) -> usize {
        let length = FnkUInt::from(self.0.len() as u64);
        let mut size = length.byte_size();

        for v in &self.0 {
            size += v.byte_size();
        }

        size
    }

    fn min_byte_size() -> usize {
        FnkUInt::min_byte_size()
    }
}

impl<'info, K: CopyType<'info> + Ord, V: CopyType<'info>> CopyType<'info> for FnkMap<K, V> {
    type ZeroCopyType = ZcFnkVec<'info, (K, V)>;

    fn byte_size(&self) -> usize {
        let length = FnkUInt::from(self.0.len() as u64);
        let mut size = length.byte_size();

        for (k, v) in &self.0 {
            size += k.byte_size();
            size += v.byte_size();
        }

        size
    }

    fn min_byte_size() -> usize {
        FnkUInt::min_byte_size()
    }
}

impl<'info, T: CopyType<'info>> ZcFnkVec<'info, T> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the vector.
    pub fn len(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let len = FnkUInt::deserialize(&mut bytes)?;

        len.get_usize()
            .ok_or_else(|| FankorErrorCode::ZeroCopyLengthFieldOverflow.into())
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

        let len = FnkUInt::deserialize(&mut bytes)?;

        let index = index as u64;
        if index >= len.0 {
            return Ok(None);
        }

        for i in 0..len.0 {
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
        let mut offset = self.offset;
        let mut length = {
            let original_bytes = (*self.info.data).borrow();
            let mut bytes = &original_bytes[self.offset..];
            let len = FnkUInt::deserialize(&mut bytes)?;

            offset += original_bytes.len() - bytes.len();

            len.get_usize()
                .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?
        };

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

        self.write_len_unchecked(FnkUInt::from(length as u64))?;

        Ok(())
    }

    pub fn iter(&self) -> Iter<'info, T> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let original_len = bytes.len();
        let len =
            FnkUInt::deserialize(&mut bytes).expect("Failed to get length of ZcFnkVec in iterator");

        Iter {
            info: self.info,
            offset: self.offset + (original_len - bytes.len()),
            len: len
                .get_usize()
                .expect("Failed to get usize from FnkUInt in iterator"),
            index: 0,
            _data: PhantomData,
        }
    }

    /// Writes the length of the vector.
    pub fn write_len_unchecked(&self, new_length: FnkUInt) -> FankorResult<()> {
        let zc = Zc::new_unchecked(self.info, self.offset);
        zc.try_write_value_unchecked(&new_length)
    }
}

impl<'info, T: CopyType<'info> + BorshSerialize> ZcFnkVec<'info, T> {
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

        // Read length.
        let length = self.len()?;
        let fnk_length = FnkUInt::from(self.len()?);
        let new_length = length
            .checked_add(values.len())
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?;
        let fnk_new_length = FnkUInt::from(new_length);

        // Append values.
        for value in values {
            let zc = Zc::new_unchecked(self.info, self.offset + size);
            let value_size = value.byte_size();
            zc.try_write_value_with_sizes_unchecked(value, 0, value_size)?;
            size += value_size;
        }

        // Write length.
        self.write_len_unchecked(fnk_new_length)?;

        let diff = fnk_new_length.byte_size() - fnk_length.byte_size();
        Ok(size + diff)
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

        // Read length.
        let length = self.len()?;
        let fnk_length = FnkUInt::from(self.len()?);
        let new_length = length
            .checked_add(values.len())
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?;
        let fnk_new_length = FnkUInt::from(new_length);

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

        // Write length.
        self.write_len_unchecked(fnk_new_length)?;

        Ok(size + fnk_new_length.byte_size() - fnk_length.byte_size())
    }
}

impl<'info, T: CopyType<'info>> IntoIterator for ZcFnkVec<'info, T> {
    type Item = Zc<'info, T>;
    type IntoIter = Iter<'info, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::create_account_info_for_tests;
    use solana_program::pubkey::Pubkey;
    use std::cell::RefCell;
    use std::mem::size_of;
    use std::rc::Rc;

    #[test]
    fn test_read_byte_length() {
        let vector = vec![2, 1, 0, 2, 0, 99];
        let size = ZcFnkVec::<u16>::read_byte_size(&vector).unwrap();

        assert_eq!(size, 1 + 2 * size_of::<u16>());
    }

    #[test]
    fn test_len_and_iter() {
        let mut lamports = 0;
        let mut vector = vec![5, 3, 3, 3, 3, 3];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcFnkVec::<u8>::new(&info, 0).unwrap();

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
    fn test_write_len() {
        let mut lamports = 0;
        let mut vector = vec![2, 3, 3, 3, 3, 3];
        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcFnkVec::<u8>::new(&info, 0).unwrap();
        zc.write_len_unchecked(FnkUInt::new(5)).unwrap();

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
        let mut vector = vec![0; 10_000];
        vector[0] = 2;
        vector[1] = 3;
        vector[2] = 3;

        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcFnkVec::<u8>::new(&info, 0).unwrap();
        let new_offset = zc.append(&[3; 500]).unwrap();

        assert_eq!(zc.len().unwrap(), 502);
        assert_eq!(new_offset, 504);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, 3);
        }

        assert_eq!(count, 502);
    }

    #[test]
    fn test_append2() {
        let mut lamports = 0;
        let mut vector = vec![0; 10_000];
        vector[0] = 2;
        vector[1] = 3;
        vector[2] = 4;
        vector[3] = 3;
        vector[4] = 4;

        let info = create_account_info_for_tests(&mut lamports, &mut vector);
        let (zc, _) = ZcFnkVec::<(u8, u8)>::new(&info, 0).unwrap();
        let data = vec![(3, 4); 500];
        let new_offset = zc.append(&data).unwrap();

        assert_eq!(zc.len().unwrap(), 502);
        assert_eq!(new_offset, 1006);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, (3, 4));
        }

        assert_eq!(count, 502);
    }

    #[test]
    fn test_append_zc() {
        let mut lamports = 0;
        let mut vector = vec![0; 100];
        vector[0] = 2;
        vector[1] = 3;
        vector[2] = 3;

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

        let (zc, _) = ZcFnkVec::<u8>::new(&info, 0).unwrap();
        let zc_el = Zc::<u8>::new_unchecked(&info_el, 0);
        let new_offset = zc.append_zc(&[zc_el.clone(), zc_el]).unwrap();

        assert_eq!(zc.len().unwrap(), 4);
        assert_eq!(new_offset, 5);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, 3);
        }

        assert_eq!(count, 4);
    }
}
