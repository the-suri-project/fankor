use crate::errors::FankorResult;
use crate::models::FankorContext;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;

/// Trait for account wrappers.
pub trait InstructionAccount<'info>: Sized {
    type CPI: CpiInstructionAccount<'info>;
    type LPI: LpiInstructionAccount;

    /// Method to get the minimum number of accounts needed to decode the instruction account.
    fn min_accounts() -> usize;

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&AccountInfo<'info>) -> FankorResult<()>;

    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self>;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait CpiInstructionAccount<'info> {
    fn to_account_metas_and_infos(
        &self,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()>;
}

impl<'info> CpiInstructionAccount<'info> for AccountInfo<'info> {
    fn to_account_metas_and_infos(
        &self,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        metas.push(AccountMeta {
            pubkey: *self.key,
            is_writable: false,
            is_signer: false,
        });
        infos.push(self.clone());
        Ok(())
    }
}

impl<'info, T: CpiInstructionAccount<'info>> CpiInstructionAccount<'info> for Option<T> {
    fn to_account_metas_and_infos(
        &self,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        if let Some(v) = self {
            v.to_account_metas_and_infos(metas, infos)?;
        }

        Ok(())
    }
}

impl<'info, T: CpiInstructionAccount<'info>> CpiInstructionAccount<'info> for Vec<T> {
    fn to_account_metas_and_infos(
        &self,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        for v in self {
            v.to_account_metas_and_infos(metas, infos)?;
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait LpiInstructionAccount {
    fn to_account_metas(&self, metas: &mut Vec<AccountMeta>) -> FankorResult<()>;
}

impl LpiInstructionAccount for solana_program::pubkey::Pubkey {
    fn to_account_metas(&self, metas: &mut Vec<AccountMeta>) -> FankorResult<()> {
        metas.push(AccountMeta {
            pubkey: *self,
            is_writable: false,
            is_signer: false,
        });
        Ok(())
    }
}

impl<T: LpiInstructionAccount> LpiInstructionAccount for Option<T> {
    fn to_account_metas(&self, metas: &mut Vec<AccountMeta>) -> FankorResult<()> {
        if let Some(v) = self {
            v.to_account_metas(metas)?;
        }

        Ok(())
    }
}

impl<T: LpiInstructionAccount> LpiInstructionAccount for Vec<T> {
    fn to_account_metas(&self, metas: &mut Vec<AccountMeta>) -> FankorResult<()> {
        for v in self {
            v.to_account_metas(metas)?;
        }

        Ok(())
    }
}
