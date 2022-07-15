use crate::errors::{ErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;

/// Same as `Vec<T>::try_from`, but limiting the maximum it can get.
pub fn try_from_vec_accounts_with_bounds<'info, T: InstructionAccount<'info>>(
    context: &'info FankorContext<'info>,
    accounts: &mut &'info [AccountInfo<'info>],
    min: usize,
    max: usize,
) -> FankorResult<Vec<T>> {
    if accounts.len() < min {
        return Err(ErrorCode::NotEnoughAccountKeys.into());
    }

    let mut result = Vec::new();
    let mut new_accounts = *accounts;

    while result.len() < max {
        if let Ok(account) = T::try_from(context, &mut new_accounts) {
            result.push(account);
        } else {
            break;
        }
    }

    if result.len() < min {
        return Err(ErrorCode::NotEnoughValidAccountForVec.into());
    }

    *accounts = new_accounts;

    Ok(result)
}
