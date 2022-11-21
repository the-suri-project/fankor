pub use fnk::*;
use std::collections::BTreeSet;

mod fnk;

use crate::errors::FankorResult;
use crate::models::zc_types::vectors::{Iter, IterMut};
use crate::models::{ZCMut, ZeroCopyType, ZC};
use borsh::BorshDeserialize;

impl<T: ZeroCopyType> ZeroCopyType for BTreeSet<T> {
    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;

        let len = self.len() as u32;
        size += len.byte_size_from_instance();

        for v in self {
            size += v.byte_size_from_instance();
        }

        size
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        let bytes2 = &mut bytes;
        let len = u32::deserialize(bytes2)?;
        size += len as usize;

        for _ in 0..len {
            bytes = &bytes[size..];
            size += T::byte_size(bytes)?;
        }

        Ok(size)
    }
}

impl<'info, 'a, T: ZeroCopyType> ZC<'info, 'a, BTreeSet<T>> {
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

    pub fn iter(&self) -> Iter<'info, 'a, T> {
        Iter {
            info: self.info,
            offset: self.offset,
            len: self
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> ZCMut<'info, 'a, BTreeSet<T>> {
    // METHODS ----------------------------------------------------------------

    pub fn iter_mut(&mut self) -> IterMut<'info, 'a, T> {
        IterMut {
            info: self.info,
            offset: self.offset,
            len: self
                .to_ref()
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> IntoIterator for ZC<'info, 'a, BTreeSet<T>> {
    type Item = ZC<'info, 'a, T>;
    type IntoIter = Iter<'info, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            info: self.info,
            offset: self.offset,
            len: self
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> IntoIterator for ZCMut<'info, 'a, BTreeSet<T>> {
    type Item = ZCMut<'info, 'a, T>;
    type IntoIter = IterMut<'info, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            info: self.info,
            offset: self.offset,
            len: self
                .to_ref()
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}
