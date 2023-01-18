use crate::ts_gen::types::{TsTypeGen, TsTypesCache};
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

impl TsTypeGen for () {
    fn value(&self) -> Cow<'static, str> {
        Cow::Borrowed("null")
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("null")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.UnitSchema")
    }
}

impl TsTypeGen for bool {
    fn value(&self) -> Cow<'static, str> {
        if *self {
            Cow::Borrowed("true")
        } else {
            Cow::Borrowed("false")
        }
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("boolean")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.Bool")
    }
}

impl TsTypeGen for i8 {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.I8")
    }
}

impl TsTypeGen for i16 {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.I16")
    }
}

impl TsTypeGen for i32 {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.I32")
    }
}

impl TsTypeGen for i64 {
    fn value(&self) -> Cow<'static, str> {
        if *self < i32::MIN as i64 || *self > i32::MAX as i64 {
            Cow::Owned(format!("new BN(\"{}\")", self))
        } else {
            Cow::Owned(format!("{}", self))
        }
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN | bigint | number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.I64")
    }
}

impl TsTypeGen for i128 {
    fn value(&self) -> Cow<'static, str> {
        if *self < i32::MIN as i128 || *self > i32::MAX as i128 {
            Cow::Owned(format!("new BN(\"{}\")", self))
        } else {
            Cow::Owned(format!("{}", self))
        }
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN | bigint | number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.I128")
    }
}

impl TsTypeGen for u8 {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.U8")
    }
}

impl TsTypeGen for u16 {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.U16")
    }
}

impl TsTypeGen for u32 {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.U32")
    }
}

impl TsTypeGen for u64 {
    fn value(&self) -> Cow<'static, str> {
        if *self > u32::MAX as u64 {
            Cow::Owned(format!("new BN(\"{}\")", self))
        } else {
            Cow::Owned(format!("{}", self))
        }
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN | bigint | number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.U64")
    }
}

impl TsTypeGen for u128 {
    fn value(&self) -> Cow<'static, str> {
        if *self > u32::MAX as u128 {
            Cow::Owned(format!("new BN(\"{}\")", self))
        } else {
            Cow::Owned(format!("{}", self))
        }
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN | bigint | number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.U128")
    }
}

impl TsTypeGen for String {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{:?}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("string")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.TString")
    }
}

impl TsTypeGen for Keypair {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "solana.Keypair.fromSeed(new Uint8Array({:?}))",
            self.secret()
        ))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.Keypair")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.TKeypair")
    }
}

impl TsTypeGen for Pubkey {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("new solana.PublicKey(\"{}\")", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.TPublicKey")
    }
}

impl<'a> TsTypeGen for &'a str {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{:?}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("string")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.TString")
    }
}

impl<T: TsTypeGen> TsTypeGen for Option<T> {
    fn value(&self) -> Cow<'static, str> {
        if let Some(v) = self {
            Cow::Owned(format!("{{ type: 'some'; value: {} }}", v.value()))
        } else {
            Cow::Borrowed("{ type: 'none' }")
        }
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!("fnk.RustOption<{}>", T::value_type()))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!("fnk.OptionSchema<{}>", T::schema_name()))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        Cow::Owned(format!("fnk.Option({})", inner_schema))
    }
}

impl<T: TsTypeGen + Any, const S: usize> TsTypeGen for [T; S] {
    fn value(&self) -> Cow<'static, str> {
        let values = self.iter().map(|v| v.value()).collect::<Vec<_>>();

        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Owned(format!("new Uint8Array([{}])", values.join(",")))
        } else {
            Cow::Owned(format!("[{}]", values.join(",")))
        }
    }

    fn value_type() -> Cow<'static, str> {
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("Uint8Array")
        } else {
            Cow::Owned(format!("{}[]", T::value_type()))
        }
    }

    fn schema_name() -> Cow<'static, str> {
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("fnk.ByteArraySchema")
        } else {
            Cow::Owned(format!("fnk.ArraySchema<{}>", T::schema_name()))
        }
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Owned(format!("fnk.ByteArray({})", S))
        } else {
            Cow::Owned(format!(
                "fnk.TArray({{ schema: {}, size: {} }})",
                inner_schema, S
            ))
        }
    }
}

impl<T: TsTypeGen + Any> TsTypeGen for Vec<T> {
    fn value(&self) -> Cow<'static, str> {
        let values = self.iter().map(|v| v.value()).collect::<Vec<_>>();

        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Owned(format!("new Uint8Array([{}])", values.join(",")))
        } else {
            Cow::Owned(format!("[{}]", values.join(",")))
        }
    }

    fn value_type() -> Cow<'static, str> {
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("Uint8Array")
        } else {
            Cow::Owned(format!("{}[]", T::value_type()))
        }
    }

    fn schema_name() -> Cow<'static, str> {
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("fnk.ByteVec")
        } else {
            Cow::Owned(format!("fnk.VecSchema<{}>", T::schema_name()))
        }
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("fnk.ByteVec")
        } else {
            Cow::Owned(format!("fnk.({})", inner_schema))
        }
    }
}

impl<T: TsTypeGen> TsTypeGen for BTreeSet<T> {
    fn value(&self) -> Cow<'static, str> {
        let values = self.iter().map(|v| v.value()).collect::<Vec<_>>();
        Cow::Owned(format!("[{}]", values.join(",")))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!("{}[]", T::value_type()))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!("fnk.VecSchema<{}>", T::schema_name()))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        Cow::Owned(format!("fnk.Vec({})", inner_schema))
    }
}

impl<K: TsTypeGen, V: TsTypeGen> TsTypeGen for BTreeMap<K, V> {
    fn value(&self) -> Cow<'static, str> {
        let values = self
            .iter()
            .map(|(k, v)| format!("{{ key: {}; value: {} }}", k.value(), v.value()))
            .collect::<Vec<_>>();

        Cow::Owned(format!("[{}]", values.join(",")))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!(
            "fnk.RustMap<{}, {}>",
            K::value_type(),
            V::value_type()
        ))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!(
            "fnk.MapSchema<{}, {}>",
            K::schema_name(),
            V::schema_name()
        ))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_key_schema = K::generate_schema(registered_schemas);
        let inner_value_schema = V::generate_schema(registered_schemas);
        Cow::Owned(format!(
            "fnk.TMap({{ keySchema: {}, valueSchema: {} }})",
            inner_key_schema, inner_value_schema
        ))
    }
}
