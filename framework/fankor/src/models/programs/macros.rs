#![allow(unused_macros)]
#![allow(unused_imports)]

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

        #[cfg(any(feature = "test-utils", test))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                let mut buf = [0u8; <$ty>::LEN];
                <$ty>::pack(self.0.clone(), &mut buf)
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

                writer.write_all(&buf)?;

                Ok(())
            }
        }

        #[cfg(not(any(feature = "test-utils", test)))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
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

    (token2022: $name: ident, $ty: ty, $owner: expr $(,)?) => {
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

        #[cfg(any(feature = "test-utils", test))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                let mut buf = [0u8; <$ty>::LEN];
                <$ty>::pack(self.0.clone(), &mut buf)
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

                writer.write_all(&buf)?;

                Ok(())
            }
        }

        #[cfg(not(any(feature = "test-utils", test)))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
            }
        }

        impl BorshDeserialize for $name {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                use spl_token_2022::extension::StateWithExtensions;
                let result = <StateWithExtensions<$ty>>::unpack(buf)
                    .map(|v| $name(v.base))
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

                // We suppose the rest bytes belong to the extensions.
                *buf = &[];

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

        #[cfg(any(feature = "test-utils", test))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                <$ty>::serialize(&self.0, writer)
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

                Ok(())
            }
        }

        #[cfg(not(any(feature = "test-utils", test)))]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
                unreachable!("Cannot write accounts that does not belong to the current program")
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
