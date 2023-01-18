use crate::models::{
    Account, CopyType, DefaultAccount, Either, Program, RefAccount, Rest, SysvarAccount,
    UncheckedAccount, UninitializedAccount, ZcAccount,
};
use crate::prelude::ProgramType;
use crate::traits::AccountType;
use crate::ts_gen::accounts::TsInstructionAccountGen;
use crate::ts_gen::types::TsTypesCache;
use solana_program::sysvar::SysvarId;
use std::borrow::Cow;

impl<'info, T: AccountType> TsInstructionAccountGen for Account<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}

impl<T: TsInstructionAccountGen> TsInstructionAccountGen for Box<T> {
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

impl<'info> TsInstructionAccountGen for DefaultAccount<'info> {
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
             else {{ accountMetas.push({{ pubkey: solana.PublicKey.default, isSigner: false, isWritable: false }}); }}",
            value, value
        ))
    }
}

impl<L: TsInstructionAccountGen, R: TsInstructionAccountGen> TsInstructionAccountGen
    for Either<L, R>
{
    fn value_type() -> Cow<'static, str> {
        let left = L::value_type();
        let right = R::value_type();

        if left == right {
            left
        } else if left.starts_with(format!("{} | ", right).as_str()) {
            left
        } else if right.starts_with(format!("{} | ", left).as_str()) {
            right
        } else {
            Cow::Owned(format!("fnk.Either<{}, {}>", left, right))
        }
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
            "if ({}.type === 'Left') {{ {} }} else {{ {} }}",
            value,
            L::get_external_account_metas(Cow::Owned(format!("{}.value", value)), signer, writable),
            R::get_external_account_metas(Cow::Owned(format!("{}.value", value)), signer, writable),
        ))
    }
}

impl<T: TsInstructionAccountGen> TsInstructionAccountGen for Option<T> {
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
            "if ({}) {{ {} }}",
            value.clone(),
            T::get_external_account_metas(value, signer, writable),
        ))
    }
}

impl<'info, T: ProgramType> TsInstructionAccountGen for Program<'info, T> {
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
            value, value, T::address()
        ))
    }
}

impl<'info, T: AccountType + 'static> TsInstructionAccountGen for RefAccount<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}

impl<'info> TsInstructionAccountGen for Rest<'info> {
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

impl<'info, T: SysvarId> TsInstructionAccountGen for SysvarAccount<'info, T> {
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

impl<'info> TsInstructionAccountGen for UncheckedAccount<'info> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}

impl<'info> TsInstructionAccountGen for UninitializedAccount<'info> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}

impl<T: TsInstructionAccountGen> TsInstructionAccountGen for Vec<T> {
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
            "{}.forEach(v => {{ {} }});",
            value,
            T::get_external_account_metas(Cow::Borrowed("v"), signer, writable)
        ))
    }
}

impl<'info, T: AccountType + CopyType<'info>> TsInstructionAccountGen for ZcAccount<'info, T> {
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("solana.PublicKey")
    }
}
