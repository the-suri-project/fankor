use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::zc_types::vec::Iter;
use crate::models::Zc;
use crate::prelude::{FnkMap, FnkUInt};
use crate::traits::{CopyType, ZeroCopyType};
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use std::marker::PhantomData;

pub struct ZcFnkMap<'info, K: CopyType<'info> + Ord, V: CopyType<'info>> {
    info: &'info AccountInfo<'info>,
    offset: usize,
    _data: PhantomData<(K, V)>,
}

impl<'info, K: CopyType<'info> + Ord, V: CopyType<'info>> ZeroCopyType<'info>
    for ZcFnkMap<'info, K, V>
{
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((
            ZcFnkMap {
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

        for _ in 0..len.0 {
            size += K::ZeroCopyType::read_byte_size(&bytes[size..])?;
            size += V::ZeroCopyType::read_byte_size(&bytes[size..])?;
        }

        Ok(size)
    }
}

impl<'info, K: CopyType<'info> + Ord, V: CopyType<'info>> CopyType<'info> for FnkMap<K, V> {
    type ZeroCopyType = ZcFnkMap<'info, K, V>;

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

impl<'info, K: CopyType<'info> + Ord, V: CopyType<'info>> ZcFnkMap<'info, K, V> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the vector.
    pub fn len(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let len = FnkUInt::deserialize(&mut bytes)?;

        Ok(len
            .get_usize()
            .ok_or(FankorErrorCode::ZeroCopyLengthFieldOverflow)?)
    }

    /// Whether the vector is empty or not
    pub fn is_empty(&self) -> FankorResult<bool> {
        Ok(self.len()? == 0)
    }

    // METHODS ----------------------------------------------------------------

    pub fn iter(&self) -> Iter<'info, (K, V)> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let original_len = bytes.len();
        let len =
            FnkUInt::deserialize(&mut bytes).expect("Failed to get length of ZcFnkMap in iterator");

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
}

impl<'info, K: CopyType<'info> + Ord, V: CopyType<'info>> IntoIterator for ZcFnkMap<'info, K, V> {
    type Item = Zc<'info, (K, V)>;
    type IntoIter = Iter<'info, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
