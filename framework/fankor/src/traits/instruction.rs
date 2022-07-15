use crate::errors::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub trait Instruction {
    fn processor(self) -> FankorResult<()>;
}

pub trait InstructionBase {
    fn discriminator() -> &'static [u8];

    fn try_parse(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> Self;

    fn base_processor(self) -> FankorResult<()>;
}
