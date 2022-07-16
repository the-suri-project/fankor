use crate::errors::FankorResult;
use crate::models::FankorContext;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

/// Trait for account wrappers.
pub trait InstructionAccount<'info>: Sized {
    type CPI: CpiInstructionAccount<'info>;

    #[cfg(feature = "library")]
    type LPI: LpiInstructionAccount<'info>;

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

#[cfg(feature = "library")]
pub trait CpiInstructionAccount<'info> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()>;
}

#[cfg(feature = "library")]
impl<'info> CpiInstructionAccount<'info> for &'info AccountInfo<'info> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()> {
        infos.push(self);
        Ok(())
    }
}

#[cfg(feature = "library")]
impl<'info, T: CpiInstructionAccount<'info>> CpiInstructionAccount<'info> for Option<T> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()> {
        if let Some(v) = self {
            v.to_account_infos(infos)?;
        }

        Ok(())
    }
}

#[cfg(feature = "library")]
impl<'info, T: CpiInstructionAccount<'info>> CpiInstructionAccount<'info> for Vec<T> {
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()> {
        for v in self {
            v.to_account_infos(infos)?;
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait LpiInstructionAccount<'info> {
    fn to_pubkeys(&self, pubkeys: &mut Vec<Pubkey>) -> FankorResult<()>;
}

impl<'info> LpiInstructionAccount<'info> for Pubkey {
    fn to_pubkeys(&self, pubkeys: &mut Vec<Pubkey>) -> FankorResult<()> {
        pubkeys.push(*self);
        Ok(())
    }
}

impl<'info, T: LpiInstructionAccount<'info>> LpiInstructionAccount<'info> for Option<T> {
    fn to_pubkeys(&self, pubkeys: &mut Vec<Pubkey>) -> FankorResult<()> {
        if let Some(v) = self {
            v.to_pubkeys(pubkeys)?;
        }

        Ok(())
    }
}

impl<'info, T: LpiInstructionAccount<'info>> LpiInstructionAccount<'info> for Vec<T> {
    fn to_pubkeys(&self, pubkeys: &mut Vec<Pubkey>) -> FankorResult<()> {
        for v in self {
            v.to_pubkeys(pubkeys)?;
        }

        Ok(())
    }
}
