use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{Account, FankorContext};
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Deserializes `T` only if the account is not the default one, i.e. 1111...1111.
///
/// This differs from `Option<T>` in that it if T does not deserialize, it does not consume
/// the account while `OptionalAccount<T>` always consumes an account, i.e. there must be a
/// deserializable account or the default one (1111...1111).
pub enum OptionalAccount<'info, T: crate::traits::AccountType> {
    Missing,
    Account(Account<'info, T>),
}

impl<'info, T: crate::traits::AccountType> OptionalAccount<'info, T> {
    // GETTERS -----------------------------------------------------------------

    pub fn is_missing(&self) -> bool {
        matches!(self, OptionalAccount::Missing)
    }

    pub fn is_account(&self) -> bool {
        !self.is_missing()
    }

    pub fn account(&self) -> Option<&Account<'info, T>> {
        match self {
            OptionalAccount::Missing => None,
            OptionalAccount::Account(v) => Some(v),
        }
    }

    pub fn account_mut(&mut self) -> Option<&mut Account<'info, T>> {
        match self {
            OptionalAccount::Missing => None,
            OptionalAccount::Account(v) => Some(v),
        }
    }

    // METHOD -----------------------------------------------------------------

    pub fn unwrap_account(self) -> Option<Account<'info, T>> {
        match self {
            OptionalAccount::Missing => None,
            OptionalAccount::Account(v) => Some(v),
        }
    }
}

impl<'info, T: crate::traits::AccountType> InstructionAccount<'info> for OptionalAccount<'info, T> {
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
            OptionalAccount::Missing => Ok(()),
            OptionalAccount::Account(v) => v.verify_account_infos(f),
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
            return Ok(OptionalAccount::Missing);
        }

        let result = OptionalAccount::Account(
            <Account<'info, T> as InstructionAccount<'info>>::try_from(context, accounts)?,
        );

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: crate::traits::AccountType> Debug for OptionalAccount<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OptionalAccount::Missing => f.debug_struct("OptionalAccount::Missing").finish(),
            OptionalAccount::Account(v) => f
                .debug_struct("OptionalAccount")
                .field("Account", &v)
                .finish(),
        }
    }
}
