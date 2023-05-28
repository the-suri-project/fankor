use std::cell::RefCell;
use std::rc::Rc;

use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

// Magic number to distinguish from normal accounts
pub const ACCOUNT_INFO_TEST_MAGIC_NUMBER: u64 = 0xAB;
const DEFAULT_PUBKEY: Pubkey = Pubkey::new_from_array([0u8; 32]);

pub fn create_account_info_for_tests<'a>(
    lamports: &'a mut u64,
    vector: &'a mut [u8],
) -> AccountInfo<'a> {
    AccountInfo {
        key: &DEFAULT_PUBKEY,
        is_signer: false,
        is_writable: false,
        lamports: Rc::new(RefCell::new(lamports)),
        data: Rc::new(RefCell::new(vector)),
        owner: &DEFAULT_PUBKEY,
        executable: false,
        rent_epoch: ACCOUNT_INFO_TEST_MAGIC_NUMBER,
    }
}
