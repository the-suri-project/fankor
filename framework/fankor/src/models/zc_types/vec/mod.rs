pub use fnk::*;

mod fnk;

use crate::errors::FankorResult;
use crate::models::{CopyType, Zc, ZeroCopyType};
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use std::marker::PhantomData;

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

    fn read_byte_size_from_bytes(mut bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        let bytes2 = &mut bytes;
        let len = u32::deserialize(bytes2)?;
        size += len as usize;

        for _ in 0..len {
            size += T::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info>> CopyType<'info> for Vec<T> {
    type ZeroCopyType = ZcVec<'info, T>;

    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;

        let len = self.len() as u32;
        size += len.byte_size_from_instance();

        for i in self {
            size += i.byte_size_from_instance();
        }

        size
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

            let size = T::ZeroCopyType::read_byte_size_from_bytes(bytes)?;
            bytes = &bytes[size..];
        }

        Ok(None)
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

        let result = Zc {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        };

        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];

        self.offset += T::ZeroCopyType::read_byte_size_from_bytes(bytes)
            .expect("Deserialization failed in vector iterator");
        self.index += 1;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;

        (size, Some(size))
    }
}

impl<'info, T: CopyType<'info>> ExactSizeIterator for Iter<'info, T> {}
