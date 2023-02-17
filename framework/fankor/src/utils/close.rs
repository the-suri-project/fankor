use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use solana_program::account_info::AccountInfo;
use solana_program::system_program;

/// Closes the `account` and sends the lamports to the `destination_account`.
pub(crate) fn close_account<'info>(
    info: &AccountInfo<'info>,
    context: &FankorContext<'info>,
    destination_account: &AccountInfo<'info>,
) -> FankorResult<()> {
    if info.owner != context.program_id() {
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

    // Transfer lamports from the account to the destination.
    **destination_account.lamports.borrow_mut() = destination_account
        .lamports()
        .checked_add(info.lamports())
        .unwrap();
    **info.lamports.borrow_mut() = 0;

    // Close the account.
    info.assign(&system_program::ID);

    #[cfg(any(feature = "test", test))]
    if info.rent_epoch != crate::tests::ACCOUNT_INFO_TEST_MAGIC_NUMBER
    {
        info.realloc(0, false)?;
    }

    #[cfg(not(any(feature = "test", test)))]
    {
        info.realloc(0, false)?;
    }

    context.remove_exit_action(info);

    Ok(())
}
