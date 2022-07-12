use solana_program::pubkey::Pubkey;
use std::mem::size_of;

/// AccountSize is a trait that allows you to determine the size of an account.
pub trait AccountSize {
    /// Returns the minimum size of an account.
    fn min_account_size() -> usize;

    /// Returns the actual size of an account.
    fn actual_account_size(&self) -> usize {
        Self::min_account_size()
    }
}

macro_rules! define_trait {
    ($($types:ty),*) => {
        $(impl AccountSize for $types {
            #[inline]
            fn min_account_size() -> usize {
                size_of::<$types>()
            }
        })*
    };
}

define_trait!(bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char, f32, f64);

impl AccountSize for Pubkey {
    #[inline]
    fn min_account_size() -> usize {
        size_of::<[u8; 32]>()
    }
}

impl AccountSize for String {
    #[inline]
    fn min_account_size() -> usize {
        size_of::<u32>() // Length
    }

    #[inline]
    fn actual_account_size(&self) -> usize {
        size_of::<u32>() // Length
        + self.as_bytes().len()
    }
}

impl<T: AccountSize> AccountSize for Vec<T> {
    #[inline]
    fn min_account_size() -> usize {
        size_of::<u32>() // Length
    }

    #[inline]
    fn actual_account_size(&self) -> usize {
        size_of::<u32>() // Length
        + self.iter().map(|x| x.actual_account_size()).sum::<usize>()
    }
}

impl<T: AccountSize> AccountSize for Option<T> {
    #[inline]
    fn min_account_size() -> usize {
        size_of::<u8>() // Discriminator
    }

    #[inline]
    fn actual_account_size(&self) -> usize {
        size_of::<u8>() // Discriminator
        + match self {
            Some(v) => v.actual_account_size(),
            None => 0
        }
    }
}

impl<T: AccountSize, const N: usize> AccountSize for [T; N] {
    #[inline]
    fn min_account_size() -> usize {
        T::min_account_size() * N
    }

    #[inline]
    fn actual_account_size(&self) -> usize {
        self.iter().map(|v| v.actual_account_size()).sum::<usize>()
    }
}
