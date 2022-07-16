use crate::accounts::EnumAccountData;
use crate::instruction::{InstructionEnumAccounts, InstructionStructAccounts};
use fankor::prelude::*;

#[program]
impl TestProgram {
    fn instruction_with_args(
        context: FankorContext,
        accounts: InstructionStructAccounts,
        arguments: EnumAccountData,
    ) -> FankorResult<()> {
        Ok(())
    }

    fn instruction_without_args(
        context: FankorContext,
        accounts: InstructionEnumAccounts,
    ) -> FankorResult<u8> {
        Ok(3)
    }

    fn fallback(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> FankorResult<()> {
        msg!("instruction1");
        Ok(())
    }
}
