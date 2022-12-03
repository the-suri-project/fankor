use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{CopyType, FankorContext};
use crate::prelude::ZcAccount;
use crate::traits::{AccountType, InstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Deserializes `Zc<T>` only if the account is not the default one, i.e. 1111...1111.
///
/// This differs from `Option<T>` in that it if T does not deserialize, it does not consume
/// the account while `OptionalZcAccount<T>` always consumes an account, i.e. there must be a
/// deserializable account or the default one (1111...1111).
pub enum OptionalZcAccount<'info, T: AccountType + CopyType<'info>> {
    Missing,
    Account(ZcAccount<'info, T>),
}

impl<'info, T: AccountType + CopyType<'info>> OptionalZcAccount<'info, T> {
    // GETTERS -----------------------------------------------------------------

    pub fn is_missing(&self) -> bool {
        matches!(self, OptionalZcAccount::Missing)
    }

    pub fn is_account(&self) -> bool {
        !self.is_missing()
    }

    pub fn account(&self) -> Option<&ZcAccount<'info, T>> {
        match self {
            OptionalZcAccount::Missing => None,
            OptionalZcAccount::Account(v) => Some(v),
        }
    }

    // METHOD -----------------------------------------------------------------

    pub fn unwrap_account(self) -> Option<ZcAccount<'info, T>> {
        match self {
            OptionalZcAccount::Missing => None,
            OptionalZcAccount::Account(v) => Some(v),
        }
    }
}

impl<'info, T: AccountType + CopyType<'info>> InstructionAccount<'info>
    for OptionalZcAccount<'info, T>
{
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    #[inline(always)]
    fn min_accounts() -> usize {
        1
    }

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&AccountInfo<'info>) -> FankorResult<()>,
    {
        match self {
            OptionalZcAccount::Missing => Ok(()),
            OptionalZcAccount::Account(v) => v.verify_account_infos(f),
        }
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(FankorErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        if info.key == &Pubkey::default() {
            *accounts = &accounts[1..];
            return Ok(OptionalZcAccount::Missing);
        }

        let result = OptionalZcAccount::Account(<ZcAccount<'info, T> as InstructionAccount<
            'info,
        >>::try_from(context, accounts)?);

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: AccountType + CopyType<'info>> Debug for OptionalZcAccount<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OptionalZcAccount::Missing => f.debug_struct("OptionalZcAccount::Missing").finish(),
            OptionalZcAccount::Account(v) => f
                .debug_struct("OptionalZcAccount")
                .field("Account", &v)
                .finish(),
        }
    }
}
