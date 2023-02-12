use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::zc_types::vec::Iter;
use crate::models::Zc;
use crate::prelude::{FnkUInt, FnkVec};
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
    pub fn append(&self, value: &[T]) -> FankorResult<()> {
        // Get current size.
        let mut size = {
            let bytes = (*self.info.data).borrow();
            let bytes = &bytes[self.offset..];
            Self::read_byte_size(bytes)?
        };

        // Update length.
        let length = self.len()?;
        let new_length = length
            .checked_add(value.len())
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?;
        self.write_len_unchecked(FnkUInt::from(new_length))?;

        // Append values.
        for v in value {
            let zc = Zc::new_unchecked(self.info, self.offset + size);
            let v_size = v.byte_size();
            zc.try_write_value_with_sizes_unchecked(v, 0, v_size)?;
            size += v_size;
        }

        Ok(())
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
        let info = AccountInfo {
            key: &Pubkey::default(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut lamports)),
            data: Rc::new(RefCell::new(&mut vector)),
            owner: &Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        };

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
        let info = AccountInfo {
            key: &Pubkey::default(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut lamports)),
            data: Rc::new(RefCell::new(&mut vector)),
            owner: &Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        };

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
        let mut vector = vec![0; 100];
        vector[0] = 2;
        vector[1] = 3;
        vector[2] = 3;

        let info = AccountInfo {
            key: &Pubkey::default(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut lamports)),
            data: Rc::new(RefCell::new(&mut vector)),
            owner: &Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        };

        let (zc, _) = ZcFnkVec::<u8>::new(&info, 0).unwrap();
        zc.append(&[3; 64]).unwrap();

        assert_eq!(zc.len().unwrap(), 66);

        let mut count = 0;
        for zc_el in zc {
            count += 1;

            let value = zc_el.try_value().unwrap();
            assert_eq!(value, 3);
        }

        assert_eq!(count, 66);
    }
}
