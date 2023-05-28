use fankor::prelude::*;
use fankor::tests::ClientExtensions;

use crate::instruction::LpiInstructionStructAccountsWithoutAssociatedType;
use crate::program::TestProgram;

#[tokio::test]
async fn test_program_instruction() {
    let mut program = TestProgram::new_program_test();
    let payer = add_payer(&mut program);

    let (mut banks_client, _payer_keypair, mut _recent_blockhash) = program.start().await;
    let instruction = crate::program::lpi::instruction_without_args(
        LpiInstructionStructAccountsWithoutAssociatedType {
            account: Pubkey::default(),
            boxed_zc_account: Pubkey::default(),
            optional_zc_account: LpiEither::Left(Pubkey::default()),
            option_zc_account: None,
            either: LpiEither::Left(Pubkey::default()),
            maybe_uninitialized: LpiEither::Left(Pubkey::default()),
            instructions_sysvar: Default::default(),
        },
    )
    .expect("Cannot build instruction");

    let transaction = banks_client
        .create_transaction_from_instructions(&[instruction], &payer, vec![&payer])
        .await
        .expect("Cannot build transaction");

    banks_client
        .process_transaction(transaction)
        .await
        .expect("Cannot process transaction");

    // Final checks
    // let account_data = banks_client
    //     .get_account_data_with_borsh(account.pubkey())
    //     .await
    //     .unwrap();
    // assert_eq!(account_data.owner, Pubkey::default());
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub fn add_payer(program: &mut ProgramTest) -> Keypair {
    let payer = Keypair::new();
    program.add_account(
        payer.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000_000_000,
            ..Default::default()
        },
    );
    payer
}
