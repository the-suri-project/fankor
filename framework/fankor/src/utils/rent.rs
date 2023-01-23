use crate::cpi;
use crate::cpi::system_program::CpiTransfer;
use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{Program, System};
use solana_program::account_info::AccountInfo;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use std::cmp::Ordering;

/// Makes an `account` be rent exempt. If `payer` is provided it ensures
/// it to be rent-exempt with only the exact required amount.
pub(crate) fn make_rent_exempt<'info>(
    new_size: usize,
    payer: &AccountInfo<'info>,
    info: &AccountInfo<'info>,
    program: &Program<System>,
) -> FankorResult<()> {
    if !payer.is_writable {
        return Err(FankorErrorCode::ReadonlyAccountModification {
            address: *payer.key,
            action: "make rent-exempt",
        }
        .into());
    }

    let original_size = unsafe { info.original_data_len() };

    if new_size == original_size {
        return Ok(());
    }

    let rent = Rent::get()?;
    let needed_balance = rent.minimum_balance(new_size);

    let current_balance = info.lamports();
    match current_balance.cmp(&needed_balance) {
        Ordering::Less => {
            let lamports = needed_balance - current_balance;

            cpi::system_program::transfer(
                program,
                CpiTransfer {
                    from: payer.clone(),
                    to: info.clone(),
                },
                lamports,
                &[],
            )
        }
        Ordering::Equal => Ok(()),
        Ordering::Greater => {
            // Transfer tokens from the account to the payer.
            let lamports = current_balance - needed_balance;

            let payer_lamports = payer.lamports();
            **payer.lamports.borrow_mut() += payer_lamports;
            **info.lamports.borrow_mut() -= lamports;

            Ok(())
        }
    }
}
