use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::CLOSED_ACCOUNT_DISCRIMINATOR;
use solana_program::account_info::AccountInfo;

/// Closes the `account` and sends the lamports to the `sol_destination`.
pub(crate) fn close_account<'info>(
    info: &AccountInfo<'info>,
    context: &FankorContext<'info>,
    sol_destination: &AccountInfo<'info>,
) -> FankorResult<()> {
    if info.key != context.program_id() {
        return Err(FankorErrorCode::AccountNotOwnedByProgram {
            address: *info.key,
            action: "close",
        }
        .into());
    }

    if !info.is_writable {
        return Err(FankorErrorCode::ReadonlyAccountModification {
            address: *info.key,
            action: "close",
        }
        .into());
    }

    // Transfer tokens from the account to the sol_destination.
    **sol_destination.lamports.borrow_mut() = sol_destination
        .lamports()
        .checked_add(info.lamports())
        .unwrap();
    **info.lamports.borrow_mut() = 0;

    // Mark the account discriminator as closed.
    let mut data = info.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    dst[0] = CLOSED_ACCOUNT_DISCRIMINATOR;

    context.set_closed_account(info, true);
    context.remove_exit_action(info);

    Ok(())
}
