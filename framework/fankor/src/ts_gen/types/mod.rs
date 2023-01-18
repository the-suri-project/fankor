pub use builtin::*;
pub use fankor::*;
use std::borrow::Cow;
use std::collections::HashMap;

mod builtin;
mod fankor;

pub type TsTypesCache = HashMap<Cow<'static, str>, Cow<'static, str>>;

pub trait TsTypeGen {
    // METHODS ----------------------------------------------------------------

    /// Gets the value of the type.
    fn value(&self) -> Cow<'static, str>;

    // STATIC METHODS ---------------------------------------------------------

    /// Gets the type of the value.
    fn value_type() -> Cow<'static, str>;

    /// Gets the schema name.
    fn schema_name() -> Cow<'static, str>;

    /// Generates the equivalent TypeScript type definition and returns the
    /// generated type name.
    #[allow(unused_variables)]
    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        Self::value_type()
    }

    /// Generates the TypeScript schema of the type and returns the expression
    /// to access the schema.
    #[allow(unused_variables)]
    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        Self::schema_name()
    }

    /// Generates the constant for the schema.
    #[allow(unused_variables)]
    fn generate_schema_constant(registered_constants: &mut TsTypesCache) {
        unreachable!("generate_schema_constant")
    }

    /// Generates the use method for the schema.
    #[allow(unused_variables)]
    fn generate_schema_use_method(registered_use_methods: &mut TsTypesCache) {
        unreachable!("generate_schema_use_method")
    }
}

impl<T: TsTypeGen> TsTypeGen for Box<T> {
    fn value(&self) -> Cow<'static, str> {
        T::value(self)
    }

    fn value_type() -> Cow<'static, str> {
        T::value_type()
    }

    fn schema_name() -> Cow<'static, str> {
        T::schema_name()
    }

    fn generate_type(
        registered_types: &mut HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> Cow<'static, str> {
        T::generate_type(registered_types)
    }

    fn generate_schema(
        registered_schemas: &mut HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> Cow<'static, str> {
        T::generate_schema(registered_schemas)
    }
}
