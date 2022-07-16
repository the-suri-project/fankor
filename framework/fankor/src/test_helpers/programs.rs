use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Default)]
pub struct ProgramHelper {
    pub program_name: Arc<Mutex<Option<&'static str>>>,
}

impl ProgramHelper {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> ProgramHelper {
        ProgramHelper::default()
    }

    // METHODS ----------------------------------------------------------------

    pub fn add_program(&self, program_name: &'static str) -> Result<(), &'static str> {
        let mut lock = self.program_name.lock();
        if let Some(item) = *lock {
            return Err(item);
        }

        *lock = Some(program_name);

        Ok(())
    }
}
