use crate::ts_gen::types::TsTypesCache;
pub use accounts::*;
use std::borrow::Cow;

mod accounts;

pub trait TsInstructionAccountGen {
    // STATIC METHODS ---------------------------------------------------------

    /// Gets the type of the account.
    fn value_type() -> Cow<'static, str>;

    /// Generates the equivalent TypeScript type definition and returns the
    /// generated type name.
    #[allow(unused_variables)]
    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        Self::value_type()
    }

    /// Generates the equivalent TypeScript resolve definition for the account.
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
}
