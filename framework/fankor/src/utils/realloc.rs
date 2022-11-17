use crate::errors::FankorResult;
use crate::models::{Program, System};
use solana_program::account_info::AccountInfo;
use crate::utils::rent::make_rent_exempt;

/// Reallocates the `account` to have at least `size` capacity.
/// If `payer` is provided it ensures it to be rent-exempt with
/// only the exact required amount.
pub(crate) fn realloc_account_to_size<'info>(
    program: &Program<System>,
    info: &AccountInfo<'info>,
    size: usize,
    zero_bytes: bool,
    payer: Option<&AccountInfo<'info>>,
) -> FankorResult<()> {
    info.realloc(size, zero_bytes)?;

    if let Some(payer) = payer {
        make_rent_exempt(program, info, size, payer)?;
    }

    Ok(())
}
