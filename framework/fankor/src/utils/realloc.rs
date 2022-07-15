use crate::errors::FankorResult;
use solana_program::account_info::AccountInfo;

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

    if let Some(_payer) = payer {
        if size > old_size {
            // TODO call CPI for system program
        }
    }

    Ok(())
}
