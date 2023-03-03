use crate::prelude::{
    FnkArray, FnkBVec, FnkExtension, FnkInt, FnkMap, FnkRange, FnkSet, FnkString, FnkUInt,
    FnkURange, FnkVec,
};
use crate::traits::{TsTypeGen, TsTypesCache};
use std::any::{Any, TypeId};
use std::borrow::Cow;

impl TsTypeGen for FnkInt {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("new BN(\"{}\")", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.FnkInt")
    }
}

impl TsTypeGen for FnkUInt {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("new BN(\"{}\")", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.FnkUInt")
    }
}

impl TsTypeGen for FnkRange {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "new fnk.FnkRange(new BN(\"{}\"), new BN(\"{}\"))",
            self.from(),
            self.to()
        ))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("fnk.FnkRange")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.TFnkRange")
    }
}

impl TsTypeGen for FnkURange {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "new fnk.FnkURange(new BN(\"{}\"), new BN(\"{}\"))",
            self.from(),
            self.to()
        ))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("fnk.FnkURange")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.TFnkURange")
    }
}

impl<'a> TsTypeGen for FnkString<'a> {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{:?}", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("string")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.FnkString")
    }
}

impl<T: TsTypeGen + Any, const S: usize> TsTypeGen for FnkArray<T, S> {
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
            let ty = T::value_type();
            Cow::Owned(format!(
                "[{}]",
                (0..S).map(|_| ty.clone()).collect::<Vec<_>>().join(",")
            ))
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

impl<T: TsTypeGen + Any> TsTypeGen for FnkVec<T> {
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
            Cow::Owned(format!("({})[]", T::value_type()))
        }
    }

    fn schema_name() -> Cow<'static, str> {
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("fnk.FnkByteVec")
        } else {
            Cow::Owned(format!("fnk.FnkVecSchema<{}>", T::schema_name()))
        }
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("fnk.FnkByteVec")
        } else {
            Cow::Owned(format!("fnk.FnkVec({})", inner_schema))
        }
    }
}

impl<T: TsTypeGen> TsTypeGen for FnkSet<T> {
    fn value(&self) -> Cow<'static, str> {
        let values = self.iter().map(|v| v.value()).collect::<Vec<_>>();
        Cow::Owned(format!("[{}]", values.join(",")))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!("({})[]", T::value_type()))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!("fnk.FnkVecSchema<{}>", T::schema_name()))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        Cow::Owned(format!("fnk.FnkVec({})", inner_schema))
    }
}

impl<K: TsTypeGen, V: TsTypeGen> TsTypeGen for FnkMap<K, V> {
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
            "fnk.FnkMapSchema<{}, {}>",
            K::schema_name(),
            V::schema_name()
        ))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_key_schema = K::generate_schema(registered_schemas);
        let inner_value_schema = V::generate_schema(registered_schemas);
        Cow::Owned(format!(
            "fnk.FnkMap({{ keySchema: {}, valueSchema: {} }})",
            inner_key_schema, inner_value_schema
        ))
    }
}

impl<K: TsTypeGen, V: TsTypeGen> TsTypeGen for FnkBVec<K, V> {
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
            "fnk.FnkBVecSchema<{}, {}>",
            K::schema_name(),
            V::schema_name()
        ))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_key_schema = K::generate_schema(registered_schemas);
        let inner_value_schema = V::generate_schema(registered_schemas);
        Cow::Owned(format!(
            "fnk.FnkBVec({{ keySchema: {}, valueSchema: {} }})",
            inner_key_schema, inner_value_schema
        ))
    }
}

impl TsTypeGen for FnkExtension {
    fn value(&self) -> Cow<'static, str> {
        Cow::Borrowed("0")
    }

    fn unit_value() -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed("0"))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("undefined")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("fnk.U8")
    }
}
