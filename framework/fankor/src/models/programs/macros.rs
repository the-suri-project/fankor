macro_rules! impl_account {
    (token: $name: ident, $ty: ty, $owner: expr $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct $name($ty);

        impl $name {
            pub fn new(data: $ty) -> Self {
                Self(data)
            }
        }

        impl crate::traits::AccountType for $name {
            fn discriminant() -> u8 {
                0
            }

            fn owner() -> &'static Pubkey {
                $owner
            }
        }

        #[cfg(any(feature = "test", test))]
        impl AccountSerialize for $name {
            fn try_serialize<W: Write>(&self, writer: &mut W) -> FankorResult<()> {
                let mut buf = [0u8; <$ty>::LEN];
                <$ty>::pack(self.0.clone(), &mut buf).map_err(|e| crate::errors::Error::from(e))?;

                writer.write_all(&buf)?;

                Ok(())
            }
        }

        #[cfg(not(any(feature = "test", test)))]
        impl AccountSerialize for $name {
            fn try_serialize<W: Write>(&self, _writer: &mut W) -> FankorResult<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
            }
        }

        #[cfg(any(feature = "test", test))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                let mut buf = [0u8; <$ty>::LEN];
                <$ty>::pack(self.0.clone(), &mut buf)
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

                writer.write_all(&buf)?;

                Ok(())
            }
        }

        #[cfg(not(any(feature = "test", test)))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
            }
        }

        impl AccountDeserialize for $name {
            fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self> {
                let result = <$ty>::unpack(buf)
                    .map($name)
                    .map_err(|e| crate::errors::Error::from(e))?;

                *buf = &buf[<$ty>::LEN..];

                Ok(result)
            }
        }

        impl BorshDeserialize for $name {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                let result = <$ty>::unpack(buf)
                    .map($name)
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

                *buf = &buf[<$ty>::LEN..];

                Ok(result)
            }
        }

        impl Deref for $name {
            type Target = $ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };

    (meta: $name: ident, $ty: ty, $owner: expr $(,)?) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct $name($ty);

        impl $name {
            pub fn new(data: $ty) -> Self {
                Self(data)
            }
        }

        impl crate::traits::AccountType for $name {
            fn discriminant() -> u8 {
                0
            }

            fn owner() -> &'static Pubkey {
                $owner
            }
        }

        #[cfg(any(feature = "test", test))]
        impl AccountSerialize for $name {
            fn try_serialize<W: Write>(&self, writer: &mut W) -> FankorResult<()> {
                <$ty>::serialize(&self.0, writer).map_err(|e| crate::errors::Error::from(e))?;

                Ok(())
            }
        }

        #[cfg(not(any(feature = "test", test)))]
        impl AccountSerialize for $name {
            fn try_serialize<W: Write>(&self, _writer: &mut W) -> FankorResult<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
            }
        }

        #[cfg(any(feature = "test", test))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                <$ty>::serialize(&self.0, writer)
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

                Ok(())
            }
        }

        #[cfg(not(any(feature = "test", test)))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
            }
        }

        impl AccountDeserialize for $name {
            fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self> {
                <$ty>::deserialize(buf)
                    .map($name)
                    .map_err(|e| crate::errors::Error::from(e))
            }
        }

        impl BorshDeserialize for $name {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                <$ty>::safe_deserialize(buf)
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
}

pub(crate) use impl_account;
