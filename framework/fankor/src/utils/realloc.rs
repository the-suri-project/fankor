use crate::cpi;
use crate::cpi::system_program::CpiTransfer;
use crate::errors::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

/// Reallocates the `account` to have at least `size` capacity.
/// If `payer` is provided it ensures it to be rent-exempt.
pub(crate) fn realloc_account_to_size<'info>(
    info: &AccountInfo<'info>,
    size: usize,
    zero_bytes: bool,
    payer: Option<&AccountInfo<'info>>,
) -> FankorResult<()> {
    let old_size = info.data_len();
    info.realloc(size, zero_bytes)?;

    if let Some(payer) = payer {
        if size > old_size {
            let rent = Rent::get()?;
            let minimum_balance = rent.minimum_balance(size);

            let current_balance = info.lamports();
            if current_balance < minimum_balance {
                let lamports = minimum_balance - current_balance;

                cpi::system_program::transfer(
                    CpiTransfer {
                        from: payer.clone(),
                        to: info.clone(),
                    },
                    lamports,
                    &[],
                )?;
            }
        }
    }

    Ok(())
}
