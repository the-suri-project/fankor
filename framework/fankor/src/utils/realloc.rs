use crate::errors::FankorResult;
use crate::models::{Program, System};
use crate::utils::rent::make_rent_exempt;
use solana_program::account_info::AccountInfo;

/// Reallocates the `account` to have at least `size` capacity.
/// If `payer` is provided it ensures it to be rent-exempt with
/// only the exact required amount.
pub(crate) fn realloc_account_to_size<'info>(
    program: &Program<System>,
    size: usize,
    zero_bytes: bool,
    info: &AccountInfo<'info>,
    payer: Option<&AccountInfo<'info>>,
) -> FankorResult<()> {
    info.realloc(size, zero_bytes)?;

    if let Some(payer) = payer {
        make_rent_exempt(program, info, size, payer)?;
    }

    Ok(())
}
