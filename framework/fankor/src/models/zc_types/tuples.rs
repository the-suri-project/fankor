use crate::errors::FankorResult;
use crate::models::{ZCMut, ZeroCopyType, ZC};
use std::marker::PhantomData;

impl<T0: ZeroCopyType, T1: ZeroCopyType> ZeroCopyType for (T0, T1) {
    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;
        size += self.0.byte_size_from_instance();
        size += self.1.byte_size_from_instance();
        size
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        size += T0::byte_size(&bytes[size..])?;
        size += T1::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info, 'a, T0: ZeroCopyType, T1: ZeroCopyType> ZC<'info, 'a, (T0, T1)> {
    // GETTERS ----------------------------------------------------------------

    pub fn to_ref_0(&self) -> FankorResult<ZC<'info, 'a, T0>> {
        Ok(ZC {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        })
    }

    pub fn to_ref_1(&self) -> FankorResult<ZC<'info, 'a, T1>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];
        let size = T0::byte_size(bytes)?;

        Ok(ZC {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        })
    }

    /// Gets the ZC of the inner values as a regular tuple.
    pub fn to_tuple(&self) -> FankorResult<(ZC<'info, 'a, T0>, ZC<'info, 'a, T1>)> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        let t0 = ZC {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        };

        let size = T0::byte_size(bytes)?;
        let t1 = ZC {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        };

        Ok((t0, t1))
    }
}

impl<'info, 'a, T0: ZeroCopyType, T1: ZeroCopyType> ZCMut<'info, 'a, (T0, T1)> {
    // GETTERS ----------------------------------------------------------------

    pub fn to_mut_0(&mut self) -> FankorResult<ZCMut<'info, 'a, T0>> {
        Ok(ZCMut {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        })
    }

    pub fn to_mut_1(&mut self) -> FankorResult<ZCMut<'info, 'a, T1>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];
        let size = T0::byte_size(bytes)?;

        Ok(ZCMut {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------


impl<T0: ZeroCopyType, T1: ZeroCopyType, T2: ZeroCopyType> ZeroCopyType for (T0, T1, T2) {
    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;
        size += self.0.byte_size_from_instance();
        size += self.1.byte_size_from_instance();
        size += self.2.byte_size_from_instance();
        size
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        size += T0::byte_size(&bytes[size..])?;
        size += T1::byte_size(&bytes[size..])?;
        size += T2::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info, 'a, T0: ZeroCopyType, T1: ZeroCopyType, T2: ZeroCopyType> ZC<'info, 'a, (T0, T1, T2)> {
    // GETTERS ----------------------------------------------------------------

    pub fn to_ref_0(&self) -> FankorResult<ZC<'info, 'a, T0>> {
        Ok(ZC {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        })
    }

    pub fn to_ref_1(&self) -> FankorResult<ZC<'info, 'a, T1>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];
        let size = T0::byte_size(bytes)?;

        Ok(ZC {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        })
    }

    pub fn to_ref_2(&self) -> FankorResult<ZC<'info, 'a, T2>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];
        let mut size = T0::byte_size(bytes)?;
        size += T1::byte_size(&bytes[size..])?;

        Ok(ZC {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        })
    }

    /// Gets the ZC of the inner values as a regular tuple.
    pub fn to_tuple(&self) -> FankorResult<(ZC<'info, 'a, T0>, ZC<'info, 'a, T1>)> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];

        let t0 = ZC {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        };

        let size = T0::byte_size(bytes)?;
        let t1 = ZC {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        };

        Ok((t0, t1))
    }
}

impl<'info, 'a, T0: ZeroCopyType, T1: ZeroCopyType, T2: ZeroCopyType> ZCMut<'info, 'a, (T0, T1, T2)> {
    // GETTERS ----------------------------------------------------------------

    pub fn to_mut_0(&mut self) -> FankorResult<ZCMut<'info, 'a, T0>> {
        Ok(ZCMut {
            info: self.info,
            offset: self.offset,
            _data: PhantomData,
        })
    }

    pub fn to_mut_1(&mut self) -> FankorResult<ZCMut<'info, 'a, T1>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];
        let size = T0::byte_size(bytes)?;

        Ok(ZCMut {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        })
    }

    pub fn to_mut_2(&mut self) -> FankorResult<ZCMut<'info, 'a, T1>> {
        let bytes = self.info.data.borrow();
        let bytes = &bytes[self.offset..];
        let mut size = T0::byte_size(bytes)?;
        size+= T1::byte_size(&bytes[size..])?;

        Ok(ZCMut {
            info: self.info,
            offset: self.offset + size,
            _data: PhantomData,
        })
    }
}
