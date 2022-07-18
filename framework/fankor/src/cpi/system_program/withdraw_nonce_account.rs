use crate::errors::Error;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiWithdrawNonceAccount<'info> {
    pub nonce: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub recent_blockhashes: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
    pub authorized: AccountInfo<'info>,
}

pub fn withdraw_nonce_account(
    accounts: CpiWithdrawNonceAccount,
    lamports: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::withdraw_nonce_account(
        accounts.nonce.key,
        accounts.authorized.key,
        accounts.to.key,
        lamports,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.nonce,
            accounts.to,
            accounts.recent_blockhashes,
            accounts.rent,
            accounts.authorized,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
