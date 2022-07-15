use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct ErrorHelper {
    pub discriminators: Arc<Mutex<HashMap<u32, ErrorHelperItem>>>,
}

#[derive(Clone)]
pub struct ErrorHelperItem {
    pub name: &'static str,
    pub discriminator: u32,
}

impl ErrorHelper {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> ErrorHelper {
        Self::default()
    }

    // METHODS ----------------------------------------------------------------

    pub fn add_error(&self, name: &'static str, discriminator: u32) -> Result<(), ErrorHelperItem> {
        let mut discriminators = self.discriminators.lock();

        if let Some(item) = discriminators.get(&discriminator) {
            return Err(item.clone());
        }

        discriminators.insert(
            discriminator,
            ErrorHelperItem {
                name,
                discriminator,
            },
        );

        Ok(())
    }
}
