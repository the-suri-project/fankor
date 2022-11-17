use crate::errors::FankorResult;
use crate::models::{ZeroCopyType, ZC};
use borsh::BorshDeserialize;

impl<T: ZeroCopyType + BorshDeserialize, const N: usize> ZeroCopyType for [T; N] {
    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;

        for v in self {
            size += v.byte_size_from_instance();
        }

        size
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        for _ in 0..N {
            bytes = &bytes[size..];
            size += T::byte_size(bytes)?;
        }

        Ok(size)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info, T: ZeroCopyType, const N: usize> ZC<'info, [T; N]> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the array.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        N
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the element at the specified position.
    pub fn get_zc_index(&self, index: usize) -> FankorResult<Option<ZC<'info, T>>> {
        if index >= N {
            return Ok(None);
        }

        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let mut size = 0;

        for i in 0..N {
            if i == index {
                return Ok(Some(ZC {
                    info: self.info,
                    offset: self.offset + size,
                    _phantom: std::marker::PhantomData,
                }));
            }

            bytes = &bytes[size..];
            size += T::byte_size(bytes)?;
        }

        Ok(None)
    }
}

impl<'info, T: ZeroCopyType, const N: usize> IntoIterator for ZC<'info, [T; N]> {
    type Item = ZC<'info, T>;
    type IntoIter = Iter<'info, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            offset: self.offset,
            len: self.len(),
            data: self,
            index: 0,
        }
    }
}

impl<'a, 'info, T: ZeroCopyType, const N: usize> IntoIterator for &'a ZC<'info, [T; N]> {
    type Item = ZC<'info, T>;
    type IntoIter = Iter<'info, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            offset: self.offset,
            data: self.clone(),
            len: self.len(),
            index: 0,
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct Iter<'info, T: ZeroCopyType, const N: usize> {
    data: ZC<'info, [T; N]>,
    len: usize,
    index: usize,
    offset: usize,
}

impl<'info, T: ZeroCopyType, const N: usize> Iterator for Iter<'info, T, N> {
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

        self.offset += T::byte_size(bytes).expect("Deserialization failed in array iterator");
        self.index += 1;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;

        (size, Some(size))
    }
}

impl<'info, T: ZeroCopyType, const N: usize> ExactSizeIterator for Iter<'info, T, N> {}
