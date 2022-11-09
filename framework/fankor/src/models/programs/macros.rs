macro_rules! impl_account {
    ($name: ident, $ty: ty, $owner: expr, $unsafe_deser_method: ident, $safe_deser_method: ident, [$($derived: tt),*]) => {
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
    };
    ($name: ident, $ty: ty, $owner: expr, $unsafe_deser_method: ident, $safe_deser_method: ident $(,)?) => {
       impl_account!($name, $ty, $owner, $unsafe_deser_method, $safe_deser_method, []);
    };
}

pub(crate) use impl_account;
