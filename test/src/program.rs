use crate::arguments::*;
use crate::errors::Errors;
use crate::instruction::*;
use fankor::prelude::*;

#[program]
impl TestProgram {
    #[validation_with_args]
    #[discriminant = 3]
    fn instruction_with_args(
        context: FankorContext<'info>,
        accounts: InstructionStructAccounts<'info>,
        arguments: InstructionArgs,
    ) -> FankorResult<()> {
        Ok(())
    }

    fn instruction_with_args2(
        context: FankorContext<'info>,
        accounts: InstructionStructAccountsWithoutAssociatedType<'info>,
        arguments: InstructionArgs,
    ) -> FankorResult<()> {
        Ok(())
    }

    #[discriminant = 5]
    fn instruction_without_args(
        context: FankorContext<'info>,
        accounts: InstructionStructAccountsWithoutAssociatedType<'info>,
    ) -> FankorResult<u8> {
        Ok(3)
    }

    fn instruction_with_enum(
        context: FankorContext<'info>,
        accounts: InstructionEnumAccountsWithoutArgs<'info>,
    ) -> FankorResult<u8> {
        Ok(3)
    }

    fn fallback(
        program_id: &'info Pubkey,
        accounts: &'info [AccountInfo],
        instruction_data: &'info [u8],
    ) -> FankorResult<()> {
        msg!("instruction1");

        require!(accounts.len() == 1, Errors::A);
        require_not!(accounts.len() == 1, Errors::A);

        Ok(())
    }
}
