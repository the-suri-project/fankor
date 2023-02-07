use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, CpiInstruction, Instruction, LpiInstruction};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::Write;

/// A wrapper around a `Vec<AccountInfo>` that keeps the rest infos.
pub struct Rest<'info> {
    context: &'info FankorContext<'info>,
    accounts: &'info [AccountInfo<'info>],
}

impl<'info> Rest<'info> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        accounts: &'info [AccountInfo<'info>],
    ) -> FankorResult<Rest<'info>> {
        Ok(Rest { context, accounts })
    }

    // GETTERS ----------------------------------------------------------------

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.accounts.len() == 0
    }

    #[inline(always)]
    pub fn accounts(&self) -> &'info [AccountInfo<'info>] {
        self.accounts
    }

    #[inline(always)]
    pub fn context(&self) -> &'info FankorContext<'info> {
        self.context
    }
}

impl<'info> Instruction<'info> for Rest<'info> {
    type CPI = CpiRest<'info>;
    type LPI = LpiRest;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        for account in self.accounts.iter() {
            config.verify(account)?;
        }

        Ok(())
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        _buf: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let result = Rest::new(context, accounts)?;

        *accounts = &[];
        Ok(result)
    }
}

impl<'info> Debug for Rest<'info> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rest")
            .field("len", &self.accounts.len())
            .finish()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiRest<'info>(pub Vec<AccountInfo<'info>>);

impl<'info> CpiRest<'info> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(accounts: Vec<AccountInfo<'info>>) -> Self {
        CpiRest(accounts)
    }
}

impl<'info> CpiInstruction<'info> for CpiRest<'info> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        for v in &self.0 {
            v.serialize_into_instruction_parts(writer, metas, infos)?;
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct LpiRest(Vec<Pubkey>);

impl LpiRest {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(accounts: Vec<Pubkey>) -> Self {
        LpiRest(accounts)
    }
}

impl LpiInstruction for LpiRest {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        for v in &self.0 {
            v.serialize_into_instruction_parts(writer, metas)?;
        }

        Ok(())
    }
}
