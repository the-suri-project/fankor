use crate::accounts::EnumAccountData;
use crate::errors::Errors;
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

        require!(accounts.len() == 1, Errors::A);
        require_not!(accounts.len() == 1, Errors::A);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instruction::LpiInstructionEnumAccounts;
    use solana_sdk::signer::Signer;
    use solana_sdk::transaction::Transaction;

    #[tokio::test]
    async fn test_x() {
        let (mut banks_client, payer, recent_blockhash) =
            TestProgram::new_program_test().start().await;

        let mut transaction = Transaction::new_with_payer(
            &[
                lpi::instruction_without_args(LpiInstructionEnumAccounts::UncheckedAccount(
                    payer.pubkey(),
                ))
                .unwrap(),
            ],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }
}
