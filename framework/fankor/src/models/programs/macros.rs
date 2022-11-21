macro_rules! impl_account {
    ($name: ident, $ty: ty, $owner: expr, $unsafe_deser_method: ident, $safe_deser_method: ident, [ZC: $size: expr], [$($derived: tt),*] $(,)?) => {
        #[derive(Debug,  Clone, PartialEq)]
        #[derive($($derived),*)]
        pub struct $name($ty);

        impl crate::traits::Account for $name {
            fn discriminator() -> u8 {
                0
            }

            fn owner() -> &'static Pubkey {
                $owner
            }
        }

        impl AccountSerialize for $name {
            fn try_serialize<W: Write>(&self, _writer: &mut W) -> FankorResult<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
            }
        }

        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
            }
        }

        impl AccountDeserialize for $name {
            unsafe fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self> {
                <$ty>::$unsafe_deser_method(buf)
                    .map($name)
                    .map_err(|e| crate::errors::Error::from(e))
            }
        }

        impl BorshDeserialize for $name {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                <$ty>::$safe_deser_method(buf)
                    .map($name)
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))
            }
        }

        impl Deref for $name {
            type Target = $ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ZeroCopyType for $name {
            fn byte_size_from_instance(&self) -> usize {
                $size
            }

            fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
                let size = $size;

                if bytes.len() < size {
                    return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                        type_name: type_name::<Self>(),
                    }
                        .into());
                }

                Ok(size)
            }
        }
    };
}

macro_rules! impl_zero_copy_account {
    ($name: path, $($ty_name: ident: $ty: ty),* $(,)?) => {
        impl<'info, 'a> ZC<'info, 'a, $name> {
            impl_zero_copy_account!(method [$($ty_name: $ty),*]);
        }
    };

    (method [$first_ty_name: ident: $first_ty: ty, $($ty_name: ident: $ty: ty),*] $($reversed_ty_name: ident: $reversed_ty: ty),* $(,)?) => {
        impl_zero_copy_account!(method [$($ty_name: $ty),*] $first_ty_name: $first_ty, $($reversed_ty_name: $reversed_ty),*);
    };

    (method [$first_ty_name: ident: $first_ty: ty] $($reversed_ty_name: ident: $reversed_ty: ty),* $(,)?) => {
        impl_zero_copy_account!(method [] $first_ty_name: $first_ty, $($reversed_ty_name: $reversed_ty),*);
    };

    (method [] $last_ty_name: ident: $last_ty: ty, $($ty_name: ident: $ty: ty),+) => {
        pub fn $last_ty_name(&self) -> FankorResult<ZC<'info, 'a, $last_ty>> {
            let bytes = (*self.info.data).borrow();
            let bytes = &bytes[self.offset..];
            let mut size = 0;

            impl_zero_copy_account!(aux size, bytes, [$($ty),*]);

            Ok(ZC {
                info: self.info,
                offset: self.offset + size,
                _data: PhantomData,
            })
        }

        impl_zero_copy_account!(method [] $($ty_name: $ty),*);
    };

    (method [] $last_ty_name: ident: $last_ty: ty) => {
        pub fn $last_ty_name(&self) -> FankorResult<ZC<'info, 'a, $last_ty>> {
            Ok(ZC {
                info: self.info,
                offset: self.offset,
                _data: PhantomData,
            })
        }
    };

    (aux $size: ident, $bytes: ident, [$first_ty: ty, $($ty: ty),*] $($reversed_ty: ty),* $(,)?) => {
        impl_zero_copy_account!(aux $size, $bytes, [$($ty),*] $first_ty, $($reversed_ty),*);
    };

    (aux $size: ident, $bytes: ident, [$first_ty: ty] $($reversed_ty: ty),* $(,)?) => {
        $size += <$first_ty>::byte_size(&$bytes[$size..])?;
        $($size += <$reversed_ty>::byte_size(&$bytes[$size..])?;)*
    };
}

pub(crate) use impl_account;
pub(crate) use impl_zero_copy_account;
