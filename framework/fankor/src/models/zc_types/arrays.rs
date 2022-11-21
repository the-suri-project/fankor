use crate::errors::FankorResult;
use crate::models::zc_types::vectors::{Iter, IterMut};
use crate::models::{ZCMut, ZeroCopyType, ZC};

impl<T: ZeroCopyType, const N: usize> ZeroCopyType for [T; N] {
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

impl<'info, 'a, T: ZeroCopyType, const N: usize> ZC<'info, 'a, [T; N]> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the array.
    pub fn len(&self) -> usize {
        N
    }

    /// Whether the array is empty or not.
    pub fn is_empty(&self) -> bool {
        N == 0
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the element at the specified position.
    pub fn get_zc_index(&self, index: usize) -> FankorResult<Option<ZC<'info, 'a, T>>> {
        if index >= N {
            return Ok(None);
        }

        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];
        let mut size = 0;

        for i in 0..N {
            if i == index {
                return Ok(Some(ZC {
                    info: self.info,
                    offset: self.offset + size,
                    _data: std::marker::PhantomData,
                }));
            }

            size += T::byte_size(&bytes[size..])?;
        }

        Ok(None)
    }

    pub fn iter(&self) -> Iter<'info, 'a, T> {
        Iter {
            info: self.info,
            offset: self.offset,
            len: N,
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType, const N: usize> ZCMut<'info, 'a, [T; N]> {
    // METHODS ----------------------------------------------------------------

    /// Gets the element at the specified position.
    pub fn get_mut_zc_index(&mut self, index: usize) -> FankorResult<Option<ZCMut<'info, 'a, T>>> {
        if index >= N {
            return Ok(None);
        }

        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];
        let mut size = 0;

        for i in 0..N {
            if i == index {
                return Ok(Some(ZCMut {
                    info: self.info,
                    offset: self.offset + size,
                    _data: std::marker::PhantomData,
                }));
            }

            size += T::byte_size(&bytes[size..])?;
        }

        Ok(None)
    }

    pub fn iter_mut(&mut self) -> IterMut<'info, 'a, T> {
        IterMut {
            info: self.info,
            offset: self.offset,
            len: N,
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType, const N: usize> IntoIterator for ZC<'info, 'a, [T; N]> {
    type Item = ZC<'info, 'a, T>;
    type IntoIter = Iter<'info, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            info: self.info,
            offset: self.offset,
            len: N,
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType, const N: usize> IntoIterator for ZCMut<'info, 'a, [T; N]> {
    type Item = ZCMut<'info, 'a, T>;
    type IntoIter = IterMut<'info, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            info: self.info,
            offset: self.offset,
            len: N,
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}
