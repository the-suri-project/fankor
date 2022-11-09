use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct InstructionHelper {
    pub discriminators: Arc<Mutex<HashMap<String, InstructionHelperItem>>>,
}

#[derive(Clone)]
pub struct InstructionHelperItem {
    pub instruction_name: &'static str,
    pub discriminator: u8,
}

impl InstructionHelper {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> InstructionHelper {
        InstructionHelper::default()
    }

    // METHODS ----------------------------------------------------------------

    pub fn add_instruction(
        &self,
        instruction_name: &'static str,
        discriminator: u8,
    ) -> Result<(), InstructionHelperItem> {
        let mut discriminators = self.discriminators.lock();
        let discriminator_str = bs58::encode(discriminator).into_string();

        if let Some(item) = discriminators.get(&discriminator_str) {
            return Err(item.clone());
        }

        discriminators.insert(
            discriminator_str,
            InstructionHelperItem {
                instruction_name,
                discriminator,
            },
        );

        Ok(())
    }
}
