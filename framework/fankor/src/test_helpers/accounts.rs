use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct AccountHelper {
    pub discriminators: Arc<Mutex<HashMap<String, AccountHelperItem>>>,
}

#[derive(Clone)]
pub struct AccountHelperItem {
    pub account_name: &'static str,
    pub discriminator: &'static [u8],
}

impl AccountHelper {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> AccountHelper {
        AccountHelper::default()
    }

    // METHODS ----------------------------------------------------------------

    pub fn add_account(
        &self,
        account_name: &'static str,
        discriminator: &'static [u8],
    ) -> Result<(), AccountHelperItem> {
        let mut discriminators = self.discriminators.lock();
        let discriminator_str = bs58::encode(discriminator).into_string();

        if let Some(item) = discriminators.get(&discriminator_str) {
            return Err(item.clone());
        }

        discriminators.insert(
            discriminator_str,
            AccountHelperItem {
                account_name,
                discriminator,
            },
        );

        Ok(())
    }
}
