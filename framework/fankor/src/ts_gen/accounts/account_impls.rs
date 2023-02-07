use crate::models::{
    Account, Argument, CopyType, Either, MaybeUninitialized, Program, Rest, SysvarAccount,
    UncheckedAccount, UninitializedAccount, ZcAccount,
};
use crate::prelude::ProgramType;
use crate::traits::AccountType;
use crate::ts_gen::accounts::TsInstructionGen;
use crate::ts_gen::types::{TsTypeGen, TsTypesCache};
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::SysvarId;
use std::borrow::Cow;

impl<'info, T: AccountType> TsInstructionGen for Account<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}

impl<T: TsTypeGen> TsInstructionGen for Argument<T> {
    fn value_type() -> Cow<'static, str> {
        T::value_type()
    }

    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        T::generate_type(registered_types)
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        _signer: bool,
        _writable: bool,
    ) -> Cow<'static, str> {
        Cow::Owned(format!(
            "{}.serialize(writer, {});",
            T::schema_name(),
            value
        ))
    }
}

impl<T: TsInstructionGen> TsInstructionGen for Box<T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }

    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        T::generate_type(registered_types)
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        signer: bool,
        writable: bool,
    ) -> Cow<'static, str> {
        T::get_external_account_metas(value, signer, writable)
    }
}

impl<L: TsInstructionGen, R: TsInstructionGen> TsInstructionGen for Either<L, R> {
    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!(
            "fnk.Either<{}, {}>",
            L::value_type(),
            R::value_type()
        ))
    }

    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        let name = Self::value_type();

        L::generate_type(registered_types);
        R::generate_type(registered_types);

        name
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        signer: bool,
        writable: bool,
    ) -> Cow<'static, str> {
        Cow::Owned(format!(
            "if ({}.type === 'Left') {{
                writer.writeByte(0);
                {}
            }} else {{
                writer.writeByte(1);
                {}
            }}",
            value,
            L::get_external_account_metas(Cow::Owned(format!("{}.value", value)), signer, writable),
            R::get_external_account_metas(Cow::Owned(format!("{}.value", value)), signer, writable),
        ))
    }
}

impl<'info, T> TsInstructionGen for MaybeUninitialized<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }

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

impl<T: TsInstructionGen> TsInstructionGen for Option<T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!("{} | null", T::value_type()))
    }

    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        let name = Self::value_type();

        T::generate_type(registered_types);

        name
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        signer: bool,
        writable: bool,
    ) -> Cow<'static, str> {
        Cow::Owned(format!(
            "if ({}) {{
                writer.writeByte(1);
                {}
            }} else {{
                writer.writeByte(0);
            }}",
            value.clone(),
            T::get_external_account_metas(value, signer, writable),
        ))
    }
}

impl<'info, T: ProgramType> TsInstructionGen for Program<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey | undefined")
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        _signer: bool,
        _writable: bool,
    ) -> Cow<'static, str> {
        let address = T::address();

        if address == &Pubkey::default() {
            Cow::Owned(format!(
                "if ({}) {{ accountMetas.push({{ pubkey: {}, isSigner: false, isWritable: false }}); }}\
                else {{ accountMetas.push({{ pubkey: solana.PublicKey.default, isSigner: false, isWritable: false }}); }}",
                value, value
            ))
        } else {
            Cow::Owned(format!(
                "if ({}) {{ accountMetas.push({{ pubkey: {}, isSigner: false, isWritable: false }}); }}\
                else {{ accountMetas.push({{ pubkey: new solana.PublicKey('{}'), isSigner: false, isWritable: false }}); }}",
                value, value, address
            ))
        }
    }
}

impl<'info> TsInstructionGen for Rest<'info> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey[]")
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        signer: bool,
        writable: bool,
    ) -> Cow<'static, str> {
        Cow::Owned(format!(
            "{}.forEach(v => {{ accountMetas.push({{ pubkey: v, isSigner: {}, isWritable: {} }}); }});",
            value, signer, writable
        ))
    }
}

impl<'info, T: SysvarId> TsInstructionGen for SysvarAccount<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey | undefined")
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        _signer: bool,
        _writable: bool,
    ) -> Cow<'static, str> {
        Cow::Owned(format!(
            "if ({}) {{ accountMetas.push({{ pubkey: {}, isSigner: false, isWritable: false }}); }}\
             else {{ accountMetas.push({{ pubkey: new solana.PublicKey('{}'), isSigner: false, isWritable: false }}); }}",
            value, value, T::id()
        ))
    }
}

impl<'info> TsInstructionGen for UncheckedAccount<'info> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}

impl<'info> TsInstructionGen for UninitializedAccount<'info> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}

impl<T: TsInstructionGen> TsInstructionGen for Vec<T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!("{}[]", T::value_type()))
    }

    fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
        let name = Self::value_type();

        T::generate_type(registered_types);

        name
    }

    fn get_account_metas(
        value: Cow<'static, str>,
        signer: bool,
        writable: bool,
    ) -> Cow<'static, str> {
        Cow::Owned(format!(
            "writer.buffer.writeUInt8({}.length, writer.length); {}.forEach(v => {{ {} }});",
            value,
            value,
            T::get_external_account_metas(Cow::Borrowed("v"), signer, writable)
        ))
    }
}

impl<'info, T: AccountType + CopyType<'info>> TsInstructionGen for ZcAccount<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}
