use crate::errors::FankorResult;
use crate::models::zc_types::vec::Iter;
use crate::models::{CopyType, Zc, ZeroCopyType};
use crate::prelude::{FnkSet, FnkUInt};
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use std::marker::PhantomData;

pub struct ZcFnkSet<'info, T: CopyType<'info>> {
    info: &'info AccountInfo<'info>,
    offset: usize,
    _data: PhantomData<T>,
}

impl<'info, T: CopyType<'info>> ZeroCopyType<'info> for ZcFnkSet<'info, T> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((
            ZcFnkSet {
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
        let len = FnkUInt::deserialize(bytes2)?;
        size += len.0 as usize;

        for _ in 0..len.0 {
            size += T::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
        }

        Ok(size)
    }
}

impl<'info, T: CopyType<'info> + Ord> CopyType<'info> for FnkSet<T> {
    type ZeroCopyType = ZcFnkSet<'info, T>;

    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;

        let len = FnkUInt::from(self.len() as u64);
        size += len.byte_size_from_instance();

        for i in &self.0 {
            size += i.byte_size_from_instance();
        }

        size
    }
}

impl<'info, T: CopyType<'info>> ZcFnkSet<'info, T> {
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

    pub fn iter(&self) -> Iter<'info, T> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let original_len = bytes.len();
        let len =
            FnkUInt::deserialize(&mut bytes).expect("Failed to get length of ZcFnkSet in iterator");

        Iter {
            info: self.info,
            offset: self.offset + (original_len - bytes.len()),
            len: len.0 as usize,
            index: 0,
            _data: PhantomData,
        }
    }
}

impl<'info, T: CopyType<'info>> IntoIterator for ZcFnkSet<'info, T> {
    type Item = Zc<'info, T>;
    type IntoIter = Iter<'info, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
