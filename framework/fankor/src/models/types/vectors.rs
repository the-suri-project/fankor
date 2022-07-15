use crate::models::types::read_length;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{ErrorKind, Write};
use std::mem::{forget, size_of};
use std::ops::{Deref, DerefMut};

pub type FnkVecU8<T> = FnkVec<T, 1>;
pub type FnkVecU16<T> = FnkVec<T, 2>;
pub type FnkVecU24<T> = FnkVec<T, 3>;

/// Wrapper over `Vec` that serializes the length into a different size.
#[derive(Debug)]
pub struct FnkVec<T, const C: usize>(pub Vec<T>);

impl<T, const C: usize> FnkVec<T, C> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: Vec<T>) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T, const C: usize> Default for FnkVec<T, C> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T, const C: usize> AsRef<Vec<T>> for FnkVec<T, C> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T, const C: usize> Deref for FnkVec<T, C> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const C: usize> DerefMut for FnkVec<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: BorshSerialize, const C: usize> BorshSerialize for FnkVec<T, C> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len_bytes = <[u8; C]>::try_from(self.len().to_le_bytes().as_slice())
            .map_err(|_| ErrorKind::InvalidInput)?;
        writer.write_all(len_bytes.as_slice())?;

        if let Some(u8_slice) = T::u8_slice(&self.0) {
            writer.write_all(u8_slice)?;
        } else {
            for item in &self.0 {
                item.serialize(writer)?;
            }
        }

        Ok(())
    }
}

impl<T: BorshDeserialize, const C: usize> BorshDeserialize for FnkVec<T, C> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let len = read_length(buf, C)?;

        if len == 0 {
            Ok(FnkVec::default())
        } else if let Some(vec_bytes) = T::vec_from_bytes(len, buf)? {
            Ok(FnkVec::new(vec_bytes))
        } else if size_of::<T>() == 0 {
            let mut result = vec![T::deserialize(buf)?];

            let p = result.as_mut_ptr();
            unsafe {
                forget(result);
                let len = len.try_into().map_err(|_| ErrorKind::InvalidInput)?;
                let result = Vec::from_raw_parts(p, len, len);
                Ok(FnkVec::new(result))
            }
        } else {
            let mut result = Vec::with_capacity({
                let el_size = size_of::<T>() as u32;
                core::cmp::max(core::cmp::min(len, 4096 / el_size), 1) as usize
            });
            for _ in 0..len {
                result.push(T::deserialize(buf)?);
            }
            Ok(FnkVec::new(result))
        }
    }
}
