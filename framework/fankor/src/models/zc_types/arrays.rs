use crate::errors::FankorResult;
use crate::models::zc_types::vec::Iter;
use crate::models::Zc;
use crate::prelude::FnkArray;
use crate::traits::{CopyType, ZeroCopyType};
use solana_program::account_info::AccountInfo;
use std::marker::PhantomData;

pub struct ZcFnkArray<'info, T: CopyType<'info>, const N: usize> {
    info: &'info AccountInfo<'info>,
    offset: usize,
    _data: PhantomData<T>,
}

impl<'info, T: CopyType<'info>, const N: usize> ZeroCopyType<'info> for ZcFnkArray<'info, T, N> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((
            ZcFnkArray {
                info,
                offset,
                _data: PhantomData,
            },
            None,
        ))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        for _ in 0..N {
            size += T::ZeroCopyType::read_byte_size(&bytes[size..])?;
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info>, const N: usize> CopyType<'info> for FnkArray<T, N> {
    type ZeroCopyType = ZcFnkArray<'info, T, N>;

    fn byte_size(&self) -> usize {
        self.iter().map(|v| v.byte_size()).sum::<usize>()
    }

    fn min_byte_size() -> usize {
        N * T::min_byte_size()
    }
}

impl<'info, T: CopyType<'info>, const N: usize> ZcFnkArray<'info, T, N> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the array.
    pub fn len(&self) -> usize {
        N
    }

    /// Whether the array is empty or not.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the element at the specified position.
    pub fn get_zc_index(&self, index: usize) -> FankorResult<Option<Zc<'info, T>>> {
        if index >= N {
            return Ok(None);
        }

        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let initial_size = bytes.len();

        for i in 0..N {
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
        Iter {
            info: self.info,
            offset: self.offset,
            len: N,
            index: 0,
            _data: PhantomData,
        }
    }
}

impl<'info, T: CopyType<'info>, const N: usize> IntoIterator for ZcFnkArray<'info, T, N> {
    type Item = Zc<'info, T>;
    type IntoIter = Iter<'info, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
