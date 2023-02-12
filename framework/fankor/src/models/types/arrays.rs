use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Debug;
use std::io::Write;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

/// Wrapper over `Arrays` that make them serializable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnkArray<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> FnkArray<T, N> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: [T; N]) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> [T; N] {
        self.0
    }
}

impl<T: Default + Copy, const N: usize> Default for FnkArray<T, N> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<T, const N: usize> AsRef<[T; N]> for FnkArray<T, N> {
    fn as_ref(&self) -> &[T; N] {
        &self.0
    }
}

impl<T, const N: usize> Deref for FnkArray<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for FnkArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const N: usize> From<[T; N]> for FnkArray<T, N> {
    fn from(v: [T; N]) -> Self {
        Self(v)
    }
}

impl<T, const N: usize> From<FnkArray<T, N>> for [T; N] {
    fn from(v: FnkArray<T, N>) -> Self {
        v.0
    }
}

impl<T: BorshSerialize, const N: usize> BorshSerialize for FnkArray<T, N> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
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

impl<T: BorshDeserialize, const N: usize> BorshDeserialize for FnkArray<T, N> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        // Copied from the last version of Borsh.
        // Remove when new version is published.
        struct ArrayDropGuard<T, const N: usize> {
            buffer: [MaybeUninit<T>; N],
            init_count: usize,
        }

        impl<T, const N: usize> Drop for ArrayDropGuard<T, N> {
            fn drop(&mut self) {
                let init_range = &mut self.buffer[..self.init_count];
                // SAFETY: Elements up to self.init_count have been initialized. Assumes this value
                //         is only incremented in `fill_buffer`, which writes the element before
                //         increasing the init_count.
                unsafe {
                    core::ptr::drop_in_place(init_range as *mut _ as *mut [T]);
                };
            }
        }

        impl<T, const N: usize> ArrayDropGuard<T, N> {
            unsafe fn transmute_to_array(mut self) -> [T; N] {
                debug_assert_eq!(self.init_count, N);
                // Set init_count to 0 so that the values do not get dropped twice.
                self.init_count = 0;
                // SAFETY: This cast is required because `mem::transmute` does not work with
                //         const generics https://github.com/rust-lang/rust/issues/61956. This
                //         array is guaranteed to be initialized by this point.
                core::ptr::read(&self.buffer as *const _ as *const [T; N])
            }

            fn fill_buffer(
                &mut self,
                mut f: impl FnMut() -> std::io::Result<T>,
            ) -> std::io::Result<()> {
                // TODO: replace with `core::array::try_from_fn` when stabilized to avoid manually
                // dropping uninitialized values through the guard drop.
                for elem in self.buffer.iter_mut() {
                    elem.write(f()?);
                    self.init_count += 1;
                }
                Ok(())
            }
        }

        let mut result = ArrayDropGuard {
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
            init_count: 0,
        };

        result.fill_buffer(|| T::deserialize(buf))?;

        // SAFETY: The elements up to `i` have been initialized in `fill_buffer`.
        Ok(FnkArray(unsafe { result.transmute_to_array() }))
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_serialize_deserialize_empty() {
        let data = [0u32; 0];
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkArray::from(data);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert!(buffer.is_empty());

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkArray::<u32, 0>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert_eq!(deserialized.0, data, "Incorrect result");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }

    #[test]
    fn test_serialize_deserialize_bytes() {
        let data = [0u8, 1, 2, 3];
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkArray::from(data);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer[0], data[0]);
        assert_eq!(buffer[1], data[1]);
        assert_eq!(buffer[2], data[2]);
        assert_eq!(buffer[3], data[3]);
        assert_eq!(buffer.len(), data.len());

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkArray::<u8, 4>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert_eq!(deserialized.0, data, "Incorrect result");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }

    #[test]
    fn test_serialize_deserialize_data() {
        let data = ["a", "b"];
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkArray::from(data);
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 0);
        assert_eq!(buffer[2], 0);
        assert_eq!(buffer[3], 0);
        assert_eq!(buffer[4], b'a');
        assert_eq!(buffer[5], 1);
        assert_eq!(buffer[6], 0);
        assert_eq!(buffer[7], 0);
        assert_eq!(buffer[8], 0);
        assert_eq!(buffer[9], b'b');
        assert_eq!(
            buffer.len(),
            data.iter().map(|v| 4 + v.len()).sum::<usize>()
        );

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkArray::<String, 2>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert_eq!(deserialized.0, data, "Incorrect result");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }
}
