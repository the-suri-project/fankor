use crate::errors::FankorResult;
use crate::models::FankorContext;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

/// Trait for account wrappers.
pub trait InstructionAccount<'info>: Sized {
    type CPI: CpiInstructionAccount<'info>;
    // type LPI: LpiInstructionAccount<'info>;

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>;

    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self>;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait CpiInstructionAccount<'info> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()>;
}

impl<'info> CpiInstructionAccount<'info> for &'info AccountInfo<'info> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()> {
        infos.push(self);
        Ok(())
    }
}

impl<'info, T: CpiInstructionAccount<'info>> CpiInstructionAccount<'info> for Option<T> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()> {
        if let Some(v) = self {
            v.to_account_infos(infos)?;
        }

        Ok(())
    }
}

impl<'info, T: CpiInstructionAccount<'info>> CpiInstructionAccount<'info> for Vec<T> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()> {
        for v in self {
            v.to_account_infos(infos)?;
        }

        Ok(())
    }
}
