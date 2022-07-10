use serde_json::{Number, Value};
use std::fmt::Write;

/// Methods all IDL types must implement.
pub trait IdlTypeMappable {
    /// Returns the IDL type.
    fn idl_type() -> IdlType;

    /// Returns the equivalent value in JSON.
    fn map_value_to_json(&self) -> Value;

    /// Returns the equivalent value in Typescript.
    fn map_value_to_typescript(&self, buffer: &mut String) {
        write!(
            buffer,
            "{}",
            serde_json::to_string(&self.map_value_to_json()).unwrap()
        )
        .unwrap();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// The different supported IDL types.
#[allow(non_camel_case_types)]
pub enum IdlType {
    Bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    Char,
    String,
    Array(Box<IdlType>),
    Tuple(Vec<IdlType>),
    Option(Box<IdlType>),
}

impl IdlType {
    /// Returns the equivalent type in JSON.
    pub fn map_type_to_json(&self) -> Value {
        match self {
            IdlType::Bool => Value::String("boolean".to_string()),
            IdlType::u8 => Value::String("u8".to_string()),
            IdlType::u16 => Value::String("u16".to_string()),
            IdlType::u32 => Value::String("u32".to_string()),
            IdlType::u64 => Value::String("u64".to_string()),
            IdlType::u128 => Value::String("u128".to_string()),
            IdlType::i8 => Value::String("i8".to_string()),
            IdlType::i16 => Value::String("i16".to_string()),
            IdlType::i32 => Value::String("i32".to_string()),
            IdlType::i64 => Value::String("i64".to_string()),
            IdlType::i128 => Value::String("i128".to_string()),
            IdlType::f32 => Value::String("f32".to_string()),
            IdlType::f64 => Value::String("f64".to_string()),
            IdlType::Char => Value::String("char".to_string()),
            IdlType::String => Value::String("string".to_string()),
            IdlType::Array(v) => {
                let mut obj = serde_json::Map::new();

                obj.insert("base".to_string(), Value::String("array".to_string()));
                obj.insert("inner".to_string(), v.map_type_to_json());

                Value::Object(obj)
            }
            IdlType::Tuple(v) => {
                let mut obj = serde_json::Map::new();

                obj.insert("base".to_string(), Value::String("tuple".to_string()));
                obj.insert(
                    "inner".to_string(),
                    Value::Array(v.iter().map(|v| v.map_type_to_json()).collect()),
                );

                Value::Object(obj)
            }
            IdlType::Option(v) => {
                let mut obj = serde_json::Map::new();

                obj.insert("base".to_string(), Value::String("option".to_string()));
                obj.insert("inner".to_string(), v.map_type_to_json());

                Value::Object(obj)
            }
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl IdlTypeMappable for () {
    fn idl_type() -> IdlType {
        IdlType::Tuple(vec![])
    }

    fn map_value_to_json(&self) -> Value {
        Value::Array(vec![])
    }
}

impl IdlTypeMappable for bool {
    fn idl_type() -> IdlType {
        IdlType::Bool
    }

    fn map_value_to_json(&self) -> Value {
        Value::Bool(*self)
    }
}

impl IdlTypeMappable for u8 {
    fn idl_type() -> IdlType {
        IdlType::u8
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for u16 {
    fn idl_type() -> IdlType {
        IdlType::u16
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for u32 {
    fn idl_type() -> IdlType {
        IdlType::u32
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for u64 {
    fn idl_type() -> IdlType {
        IdlType::u64
    }

    fn map_value_to_json(&self) -> Value {
        Value::String(self.to_string())
    }

    fn map_value_to_typescript(&self, buffer: &mut String) {
        write!(buffer, "new BN(\"{}\")", self).unwrap();
    }
}

impl IdlTypeMappable for u128 {
    fn idl_type() -> IdlType {
        IdlType::u128
    }

    fn map_value_to_json(&self) -> Value {
        Value::String(self.to_string())
    }

    fn map_value_to_typescript(&self, buffer: &mut String) {
        write!(buffer, "new BN(\"{}\")", self).unwrap();
    }
}

impl IdlTypeMappable for i8 {
    fn idl_type() -> IdlType {
        IdlType::i8
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for i16 {
    fn idl_type() -> IdlType {
        IdlType::i16
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for i32 {
    fn idl_type() -> IdlType {
        IdlType::i32
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for i64 {
    fn idl_type() -> IdlType {
        IdlType::i64
    }

    fn map_value_to_json(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl IdlTypeMappable for i128 {
    fn idl_type() -> IdlType {
        IdlType::i128
    }

    fn map_value_to_json(&self) -> Value {
        Value::String(self.to_string())
    }

    fn map_value_to_typescript(&self, buffer: &mut String) {
        write!(buffer, "new BN(\"{}\")", self).unwrap();
    }
}

impl IdlTypeMappable for f32 {
    fn idl_type() -> IdlType {
        IdlType::f32
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from_f64(*self as f64).unwrap())
    }
}

impl IdlTypeMappable for f64 {
    fn idl_type() -> IdlType {
        IdlType::f64
    }

    fn map_value_to_json(&self) -> Value {
        Value::Number(Number::from_f64(*self as f64).unwrap())
    }
}

impl IdlTypeMappable for char {
    fn idl_type() -> IdlType {
        IdlType::Char
    }

    fn map_value_to_json(&self) -> Value {
        Value::String(format!("{}", *self))
    }
}

impl IdlTypeMappable for String {
    fn idl_type() -> IdlType {
        IdlType::String
    }

    fn map_value_to_json(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl IdlTypeMappable for &str {
    fn idl_type() -> IdlType {
        IdlType::String
    }

    fn map_value_to_json(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for Option<T> {
    fn idl_type() -> IdlType {
        IdlType::Option(Box::new(T::idl_type()))
    }

    fn map_value_to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();

        obj.insert(
            "isNull".to_string(),
            Value::String(self.is_some().to_string()),
        );

        if let Some(v) = self {
            obj.insert("value".to_string(), v.map_value_to_json());
        }

        Value::Object(obj)
    }
}

impl<T: IdlTypeMappable, const N: usize> IdlTypeMappable for [T; N] {
    fn idl_type() -> IdlType {
        IdlType::Array(Box::new(T::idl_type()))
    }

    fn map_value_to_json(&self) -> Value {
        Value::Array(self.iter().map(|v| v.map_value_to_json()).collect())
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for [T] {
    fn idl_type() -> IdlType {
        IdlType::Array(Box::new(T::idl_type()))
    }

    fn map_value_to_json(&self) -> Value {
        Value::Array(self.iter().map(|v| v.map_value_to_json()).collect())
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for Vec<T> {
    fn idl_type() -> IdlType {
        IdlType::Array(Box::new(T::idl_type()))
    }

    fn map_value_to_json(&self) -> Value {
        Value::Array(self.iter().map(|v| v.map_value_to_json()).collect())
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for &T {
    fn idl_type() -> IdlType {
        T::idl_type()
    }

    fn map_value_to_json(&self) -> Value {
        T::map_value_to_json(self)
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for &mut T {
    fn idl_type() -> IdlType {
        T::idl_type()
    }

    fn map_value_to_json(&self) -> Value {
        T::map_value_to_json(self)
    }
}

macro_rules! build_tuple_impls {
    ($($id:tt: $param:ident),+) => {
        build_tuple_impls!(@ [] [$($id: $param)+]);
    };
    (@ [$($idFirstList:tt: $paramFirstList:ident)*] [$idFirst:tt: $paramFirst:ident $($id:tt: $param:ident)*]) => {
       build_tuple_impls!(@build $($idFirstList: $paramFirstList)*);
       build_tuple_impls!(@ [$($idFirstList: $paramFirstList)* $idFirst: $paramFirst] [$($id: $param)*]);
    };
    (@ [$($id:tt: $param:ident)*] []) => {
       build_tuple_impls!(@build $($id: $param)*);
    };
    (@build $($id:tt: $param:ident)+) => {
        impl<$($param: IdlTypeMappable),+> IdlTypeMappable for ($($param),+,) {
            fn idl_type() -> IdlType {
                IdlType::Tuple(vec![$($param::idl_type()),*])
            }

            fn map_value_to_json(&self) -> Value {
                serde_json::Value::Array(vec![$(build_tuple_impls!(@expr self.$id).map_value_to_json()),*])
            }
        }
    };
    (@build) => {};

    // Hack
    (@expr $x:expr) => {
        ($x)
    };
}

build_tuple_impls!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
