use serde_json::{Number, Value};

/// Methods all IDL types must implement.
pub trait IdlTypeMappable {
    /// Returns the IDL type.
    fn idl_type() -> IdlType;

    /// Returns the value as an IDL string.
    fn map_to_idl(&self) -> Value;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// The different supported IDL types.
pub enum IdlType {
    Bool,
    Number,
    BigNumber,
    String,
    Array(Box<IdlType>),
    Tuple(Vec<IdlType>),
    Option(Box<IdlType>),
}

impl IdlType {
    pub fn to_idl_string(&self) -> Value {
        let mut obj = serde_json::Map::new();

        match self {
            IdlType::Bool => {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("\"boolean\"".to_string()),
                );
            }
            IdlType::Number => {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("\"number\"".to_string()),
                );
            }
            IdlType::BigNumber => {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("\"bigNumber\"".to_string()),
                );
            }
            IdlType::String => {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("\"string\"".to_string()),
                );
            }
            IdlType::Array(v) => {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("\"array\"".to_string()),
                );
                obj.insert("inner".to_string(), v.to_idl_string());
            }
            IdlType::Tuple(v) => {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("\"tuple\"".to_string()),
                );
                obj.insert(
                    "inner".to_string(),
                    serde_json::Value::Array(v.iter().map(|v| v.to_idl_string()).collect()),
                );
            }
            IdlType::Option(v) => {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("\"option\"".to_string()),
                );
                obj.insert("inner".to_string(), v.to_idl_string());
            }
        }

        serde_json::Value::Object(obj)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl IdlTypeMappable for () {
    fn idl_type() -> IdlType {
        IdlType::Tuple(vec![])
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Array(vec![])
    }
}

impl IdlTypeMappable for bool {
    fn idl_type() -> IdlType {
        IdlType::Bool
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Bool(*self)
    }
}

impl IdlTypeMappable for u8 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for u16 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for u32 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for u64 {
    fn idl_type() -> IdlType {
        IdlType::BigNumber
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::String(self.to_string())
    }
}

impl IdlTypeMappable for i8 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for i16 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for i32 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Number(Number::from(*self))
    }
}

impl IdlTypeMappable for i64 {
    fn idl_type() -> IdlType {
        IdlType::BigNumber
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::String(self.to_string())
    }
}

impl IdlTypeMappable for f32 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::String(self.to_string())
    }
}

impl IdlTypeMappable for f64 {
    fn idl_type() -> IdlType {
        IdlType::Number
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::String(self.to_string())
    }
}

impl IdlTypeMappable for char {
    fn idl_type() -> IdlType {
        IdlType::String
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::String(self.to_string())
    }
}

impl IdlTypeMappable for String {
    fn idl_type() -> IdlType {
        IdlType::String
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::String(self.to_string())
    }
}

impl IdlTypeMappable for &str {
    fn idl_type() -> IdlType {
        IdlType::String
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::String(self.to_string())
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for Option<T> {
    fn idl_type() -> IdlType {
        IdlType::Option(Box::new(T::idl_type()))
    }

    fn map_to_idl(&self) -> Value {
        match self {
            Some(v) => v.map_to_idl(),
            None => serde_json::Value::Null,
        }
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for [T] {
    fn idl_type() -> IdlType {
        IdlType::Array(Box::new(T::idl_type()))
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Array(self.iter().map(|v| v.map_to_idl()).collect())
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for Vec<T> {
    fn idl_type() -> IdlType {
        IdlType::Array(Box::new(T::idl_type()))
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Array(self.iter().map(|v| v.map_to_idl()).collect())
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for &T {
    fn idl_type() -> IdlType {
        T::idl_type()
    }

    fn map_to_idl(&self) -> Value {
        T::map_to_idl(self)
    }
}

impl<T: IdlTypeMappable> IdlTypeMappable for &mut T {
    fn idl_type() -> IdlType {
        T::idl_type()
    }

    fn map_to_idl(&self) -> Value {
        T::map_to_idl(self)
    }
}

impl<A: IdlTypeMappable> IdlTypeMappable for (A,) {
    fn idl_type() -> IdlType {
        IdlType::Tuple(vec![A::idl_type()])
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Array(vec![self.0.map_to_idl()])
    }
}

impl<A: IdlTypeMappable, B: IdlTypeMappable> IdlTypeMappable for (A, B) {
    fn idl_type() -> IdlType {
        IdlType::Tuple(vec![A::idl_type(), B::idl_type()])
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Array(vec![self.0.map_to_idl(), self.1.map_to_idl()])
    }
}

impl<A: IdlTypeMappable, B: IdlTypeMappable, C: IdlTypeMappable> IdlTypeMappable for (A, B, C) {
    fn idl_type() -> IdlType {
        IdlType::Tuple(vec![A::idl_type(), B::idl_type(), C::idl_type()])
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Array(vec![
            self.0.map_to_idl(),
            self.1.map_to_idl(),
            self.2.map_to_idl(),
        ])
    }
}

impl<A: IdlTypeMappable, B: IdlTypeMappable, C: IdlTypeMappable, D: IdlTypeMappable> IdlTypeMappable
    for (A, B, C, D)
{
    fn idl_type() -> IdlType {
        IdlType::Tuple(vec![
            A::idl_type(),
            B::idl_type(),
            C::idl_type(),
            D::idl_type(),
        ])
    }

    fn map_to_idl(&self) -> Value {
        serde_json::Value::Array(vec![
            self.0.map_to_idl(),
            self.1.map_to_idl(),
            self.2.map_to_idl(),
            self.3.map_to_idl(),
        ])
    }
}
