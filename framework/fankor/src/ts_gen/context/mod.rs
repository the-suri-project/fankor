pub use data::*;

mod data;

use crate::ts_gen::context::data::DataContext;
use std::panic::UnwindSafe;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fs, thread};

/// Contains helper data to do the building process.
pub struct BuildContext {
    finished: AtomicBool,
    actions: Arc<Mutex<Vec<BuildAction>>>,
    start_at: AtomicU64,
    mutex: Arc<Mutex<DataContext>>,
}

impl BuildContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new [BuildContext].
    pub fn new() -> BuildContext {
        BuildContext {
            finished: AtomicBool::new(false),
            actions: Arc::new(Mutex::new(Vec::new())),
            start_at: AtomicU64::new({
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                now + 1
            }),
            mutex: Arc::new(Mutex::new(DataContext::new())),
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the finished flag.
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    /// Gets a mutable reference to the data data.
    pub fn data_context(&self) -> MutexGuard<DataContext> {
        self.mutex.lock().unwrap()
    }

    /// Gets the remaining number of actions.
    pub fn remaining_actions(&self) -> usize {
        self.actions.lock().unwrap().len()
    }

    // METHODS ----------------------------------------------------------------

    /// Finishes the building process setting the finished flag.
    pub fn finish(&self) {
        self.finished.store(true, Ordering::SeqCst);
    }

    /// Adds an action to the context.
    pub fn register_action<F>(&self, test_name: &'static str, file_path: &'static str, function: F)
    where
        F: 'static + FnOnce(&mut DataContext) + UnwindSafe + Send,
    {
        if self.finished() {
            panic!("Race error: the task has been registered after starting the building process.");
        }

        let mut actions = self.actions.lock().unwrap();
        println!("Registering action[{}]: {}", actions.len(), test_name);

        // Increase time to make the main thread wait.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.start_at.store(now + 1, Ordering::SeqCst);

        actions.push(BuildAction {
            test_name,
            file_path,
            function: Box::new(function),
        });
    }

    /// Pops an action from the context.
    pub fn pop_action(&self) -> Option<BuildAction> {
        let mut actions = self.actions.lock().unwrap();
        actions.pop()
    }

    /// Performs the build process waiting 4 seconds till all tests register their actions.
    pub fn build(&self) {
        // Wait till start_at.
        loop {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let start_at = self.start_at.load(Ordering::SeqCst);

            if now >= start_at {
                break;
            }

            thread::sleep(Duration::from_secs(start_at - now));
        }

        let mut total_actions = 0;
        let data_context = self.execute_actions(self.data_context(), &mut total_actions);
        println!("{} actions done [first round]", total_actions);

        // Finish the development process.
        self.finish();

        // Execute again just in case during the execution of the finishing action
        // another one was registered.
        total_actions = 0;

        let idl_build_context = self.execute_actions(data_context, &mut total_actions);
        println!("{} actions done [second round]", total_actions);

        // Generate the IDL files.
        self.generate(idl_build_context);
        println!("IDL generation done.");
    }

    fn execute_actions<'a>(
        &self,
        mut data_context: MutexGuard<'a, DataContext>,
        total_actions: &mut usize,
    ) -> MutexGuard<'a, DataContext> {
        while let Some(action) = self.pop_action() {
            println!("Executing action: {}", action.test_name);
            // Executes the function and captures any error to report it to others.
            (action.function)(&mut data_context);
            *total_actions += 1;
        }

        data_context
    }

    fn generate(&self, mut data_context: MutexGuard<DataContext>) {
        let folder_path = format!("target/fnk_ts");
        let file_path = format!("{}/{}.ts", folder_path, data_context.program_name);

        // Remove file.
        let _ = fs::remove_file(file_path.as_str());

        // Create folder.
        fs::create_dir_all(folder_path.as_str())
            .unwrap_or_else(|e| panic!("Cannot create folder '{}': {}", folder_path, e));

        // Generate the TypeScript file.
        let file_content = data_context.build_ts_file();
        fs::write(file_path.as_str(), file_content.as_str())
            .unwrap_or_else(|e| panic!("Cannot write file '{}': {}", file_path, e));
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A registered build action.
pub struct BuildAction {
    pub test_name: &'static str,
    pub file_path: &'static str,
    pub function: Box<dyn FnOnce(&mut DataContext) + UnwindSafe + Send>,
}
