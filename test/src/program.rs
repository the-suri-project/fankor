use crate::accounts::EnumAccountData;
use crate::errors::Errors;
use crate::instruction::{
    InstructionStructAccounts, InstructionStructAccountsWithoutAssociatedType,
};
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

    #[independent_validation]
    fn instruction_with_args2(
        context: FankorContext,
        accounts: InstructionStructAccountsWithoutAssociatedType,
        arguments: EnumAccountData,
    ) -> FankorResult<()> {
        Ok(())
    }

    fn instruction_without_args(
        context: FankorContext,
        accounts: InstructionStructAccountsWithoutAssociatedType,
    ) -> FankorResult<u8> {
        Ok(3)
    }

    fn fallback(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> FankorResult<()> {
        msg!("instruction1");

        require!(accounts.len() == 1, Errors::A);
        require_not!(accounts.len() == 1, Errors::A);

        Ok(())
    }
}
