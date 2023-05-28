use std::cmp::Ordering;

use solana_program::account_info::AccountInfo;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::cpi;
use crate::cpi::system_program::CpiTransfer;
use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{Program, System};

/// Makes an `account` be rent exempt. If `exact` is provided it ensures
/// it to be rent-exempt with only the exact required amount, i.e.
/// decreasing the account balance if needed.
pub(crate) fn make_rent_exempt<'info>(
    new_size: usize,
    exact: bool,
    payer: &AccountInfo<'info>,
    info: &AccountInfo<'info>,
    system_program: &Program<System>,
) -> FankorResult<()> {
    if !payer.is_writable {
        return Err(FankorErrorCode::ReadonlyAccountModification {
            address: *payer.key,
            action: "make rent-exempt",
        }
        .into());
    }

    if !info.is_writable {
        return Err(FankorErrorCode::ReadonlyAccountModification {
            address: *info.key,
            action: "make rent-exempt",
        }
        .into());
    }

    let rent = Rent::get()?;
    let needed_balance = rent.minimum_balance(new_size);
    let current_balance = info.lamports();

    match current_balance.cmp(&needed_balance) {
        Ordering::Less => {
            // Transfer funds to the account.
            let lamports = needed_balance - current_balance;

            cpi::system_program::transfer(
                system_program,
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
            if !exact {
                return Ok(());
            }

            // Transfer tokens from the account to the payer.
            let lamports = current_balance - needed_balance;

            let payer_lamports = payer.lamports();
            let info_lamports = info.lamports();

            **info.lamports.borrow_mut() = info_lamports.checked_sub(lamports).unwrap();
            **payer.lamports.borrow_mut() = payer_lamports.checked_add(lamports).unwrap();

            // Note: manually changing the lamports of the accounts can cause problems if any of
            // those accounts are used in a CPI after these modifications but not both at the same
            // time. To fix it, we do a NOOP operation over them in order to make the runtime know
            // about the changes.
            //
            // This is a simple workaround that only works if payer is signer, in any other case
            // the fix must be applied manually.
            if payer.is_signer {
                cpi::system_program::transfer(
                    system_program,
                    CpiTransfer {
                        from: payer.clone(),
                        to: info.clone(),
                    },
                    0,
                    &[],
                )?;
            }

            Ok(())
        }
    }
}
