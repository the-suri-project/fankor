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
    #[inline(always)]
    fn min_account_size() -> usize {
        size_of::<[u8; 32]>()
    }
}

impl AccountSize for String {
    #[inline(always)]
    fn min_account_size() -> usize {
        size_of::<u32>() // Length
    }

    #[inline]
    fn actual_account_size(&self) -> usize {
        size_of::<u32>() // Length
        + self.as_bytes().len()
    }
}

impl<T: AccountSize> AccountSize for Box<T> {
    #[inline(always)]
    fn min_account_size() -> usize {
        // Prevents infinite recursion.
        0
    }

    #[inline]
    fn actual_account_size(&self) -> usize {
        T::actual_account_size(&**self)
    }
}

impl<T: AccountSize> AccountSize for Vec<T> {
    #[inline(always)]
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
    #[inline(always)]
    fn min_account_size() -> usize {
        size_of::<u8>() // Discriminant
    }

    #[inline]
    fn actual_account_size(&self) -> usize {
        size_of::<u8>() // Discriminant
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

impl AccountSize for () {
    #[inline(always)]
    fn min_account_size() -> usize {
        0
    }
}

macro_rules! define_tuple_trait {
    ($($types:ident),*) => {
        impl< $($types : AccountSize),* > AccountSize for ($($types),*) {
            #[inline]
            fn min_account_size() -> usize {
                let mut size = 0;

                $(
                    size += <$types>::min_account_size();
                )*

                size
            }
        }
    };
}

define_tuple_trait!(T0, T1);
define_tuple_trait!(T0, T1, T2);
define_tuple_trait!(T0, T1, T2, T3);
define_tuple_trait!(T0, T1, T2, T3, T4);
define_tuple_trait!(T0, T1, T2, T3, T4, T5);
define_tuple_trait!(T0, T1, T2, T3, T4, T5, T6);
define_tuple_trait!(T0, T1, T2, T3, T4, T5, T6, T7);
define_tuple_trait!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
define_tuple_trait!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
define_tuple_trait!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
