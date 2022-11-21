use crate::errors::FankorResult;
use crate::models::zc_types::vectors::{Iter, IterMut};
use crate::models::{ZCMut, ZeroCopyType, ZC};
use crate::prelude::{FnkSet, FnkUInt};
use borsh::BorshDeserialize;

impl<T: ZeroCopyType> ZeroCopyType for FnkSet<T> {
    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;

        let len = FnkUInt::from(self.len() as u64);
        size += len.byte_size_from_instance();

        for v in &self.0 {
            size += v.byte_size_from_instance();
        }

        size
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        let bytes2 = &mut bytes;
        let len = FnkUInt::deserialize(bytes2)?;
        size += len.0 as usize;

        for _ in 0..len.0 {
            bytes = &bytes[size..];
            size += T::byte_size(bytes)?;
        }

        Ok(size)
    }
}

impl<'info, 'a, T: ZeroCopyType> ZC<'info, 'a, FnkSet<T>> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the vector.
    pub fn len(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let len = FnkUInt::deserialize(&mut bytes)?;

        Ok(len.0 as usize)
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

impl<'info, 'a, T: ZeroCopyType> ZCMut<'info, 'a, FnkSet<T>> {
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

impl<'info, 'a, T: ZeroCopyType> IntoIterator for ZC<'info, 'a, FnkSet<T>> {
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

impl<'info, 'a, T: ZeroCopyType> IntoIterator for ZCMut<'info, 'a, FnkSet<T>> {
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
