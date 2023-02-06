use crate::ts_gen::types::TsTypesCache;
pub use account_impls::*;
use std::borrow::Cow;

mod account_impls;

pub trait TsInstructionGen {
    // STATIC METHODS ---------------------------------------------------------

    /// Gets the type of the account.
    fn value_type() -> Cow<'static, str>;

    /// Generates the equivalent TypeScript type definition and returns the
    /// generated type name.
    #[allow(unused_variables)]
    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        Self::value_type()
    }

    /// Generates the code to include the account metas of the type in the
    /// getMetasOf method.
    #[allow(unused_variables)]
    fn get_account_metas(
        value: Cow<'static, str>,
        signer: bool,
        writable: bool,
    ) -> Cow<'static, str> {
        Cow::Owned(format!(
            "accountMetas.push({{ pubkey: {}, isSigner: {}, isWritable: {} }});",
            value, signer, writable
        ))
    }

    /// Generates the code to include the account metas of the type in another
    /// account's getMetasOf method. This must return the getMetasOf method.
    #[allow(unused_variables)]
    fn get_external_account_metas(
        value: Cow<'static, str>,
        signer: bool,
        writable: bool,
    ) -> Cow<'static, str> {
        Self::get_account_metas(value, signer, writable)
    }
}
