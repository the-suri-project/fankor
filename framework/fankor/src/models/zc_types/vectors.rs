use crate::errors::FankorResult;
use crate::models::{ZeroCopyType, ZC};
use crate::prelude::{FnkUInt, FnkVec};
use borsh::BorshDeserialize;

impl<T: ZeroCopyType> ZeroCopyType for FnkVec<T> {
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

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info, T: ZeroCopyType> ZC<'info, FnkVec<T>> {
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

    /// Gets the element at the specified position.
    pub fn get_zc_index(&self, index: usize) -> FankorResult<Option<ZC<'info, T>>> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let initial_size = bytes.len();

        let len = FnkUInt::deserialize(&mut bytes)?;

        let index = index as u64;
        if index as u64 >= len.0 {
            return Ok(None);
        }

        for i in 0..len.0 {
            if i == index {
                return Ok(Some(ZC {
                    info: self.info,
                    offset: self.offset + initial_size - bytes.len(),
                    _phantom: std::marker::PhantomData,
                }));
            }

            let size = T::byte_size(bytes)?;
            bytes = &bytes[size..];
        }

        Ok(None)
    }
}

impl<'info, T: ZeroCopyType> IntoIterator for ZC<'info, FnkVec<T>> {
    type Item = ZC<'info, T>;
    type IntoIter = Iter<'info, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            offset: self.offset,
            len: self
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            data: self,
            index: 0,
        }
    }
}

impl<'a, 'info, T: ZeroCopyType> IntoIterator for &'a ZC<'info, FnkVec<T>> {
    type Item = ZC<'info, T>;
    type IntoIter = Iter<'info, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            offset: self.offset,
            data: self.clone(),
            len: self
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct Iter<'info, T: ZeroCopyType> {
    data: ZC<'info, FnkVec<T>>,
    len: usize,
    index: usize,
    offset: usize,
}

impl<'info, T: ZeroCopyType> Iterator for Iter<'info, T> {
    type Item = ZC<'info, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let result = ZC {
            info: self.data.info,
            offset: self.offset,
            _phantom: std::marker::PhantomData,
        };

        let bytes = (*self.data.info.data).borrow();
        let bytes = &bytes[self.offset..];

        self.offset += T::byte_size(bytes).expect("Deserialization failed in vector iterator");
        self.index += 1;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;

        (size, Some(size))
    }
}

impl<'info, T: ZeroCopyType> ExactSizeIterator for Iter<'info, T> {}
