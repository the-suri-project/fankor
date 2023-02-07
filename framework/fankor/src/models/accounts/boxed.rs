use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, Instruction, PdaChecker, SingleInstructionAccount};
use solana_program::account_info::AccountInfo;

impl<'info, T: Instruction<'info>> Instruction<'info> for Box<T> {
    type CPI = AccountInfo<'info>;
    type LPI = solana_program::pubkey::Pubkey;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        let value: &T = self;
        T::verify_account_infos(value, config)
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        buf: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        Ok(Box::new(T::try_from(context, buf, accounts)?))
    }
}

impl<'info, T: SingleInstructionAccount<'info>> SingleInstructionAccount<'info> for Box<T> {}

impl<'info, T: PdaChecker<'info>> PdaChecker<'info> for Box<T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        let aux: &T = self;
        aux.pda_info()
    }
}
