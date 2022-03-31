use std::panic::{resume_unwind, AssertUnwindSafe, UnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use std::{fs, thread};

use crate::build::{FankorConfig, IdlBuildContext};

/// Contains helper data to do the building process.
pub struct IdlContext {
    finished: AtomicBool,
    actions: Arc<Mutex<Vec<IdlAction>>>,
    mutex: Arc<Mutex<IdlBuildContext>>,
}

impl IdlContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new IDL build context.
    pub fn new() -> IdlContext {
        IdlContext {
            finished: AtomicBool::new(false),
            actions: Arc::new(Mutex::new(Vec::new())),
            mutex: Arc::new(Mutex::new(IdlBuildContext::new())),
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the finished flag.
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    /// Gets a mutable reference to the IDL build context.
    pub fn build_context(&self) -> MutexGuard<IdlBuildContext> {
        self.mutex.lock().unwrap()
    }

    /// Gets the remaining number of actions.
    pub fn remaining_actions(&self) -> usize {
        self.actions.lock().unwrap().len()
    }

    // METHODS ----------------------------------------------------------------

    /// Finishes the IDL building process setting the finished flag.
    pub fn finish(&self) {
        self.finished.store(true, Ordering::SeqCst);
    }

    /// Adds an action to the context.
    pub fn register_action<F>(&self, test_name: &'static str, file_path: &'static str, function: F)
    where
        F: 'static + FnOnce(&mut IdlBuildContext) -> () + UnwindSafe + Send,
    {
        if self.finished() {
            panic!("Race error: the task has not been registered before starting the building process.");
        }

        let mut actions = self.actions.lock().unwrap();
        actions.push(IdlAction {
            test_name,
            file_path,
            function: Box::new(function),
        });
    }

    /// Pops an action from the context.
    pub fn pop_action(&self) -> Option<IdlAction> {
        let mut actions = self.actions.lock().unwrap();
        actions.pop()
    }

    /// Performs the build process.
    pub fn build(&self) {
        // Read config.
        let config = match std::fs::read_to_string("./Fankor.toml") {
            Ok(file_content) => match toml::from_str::<FankorConfig>(file_content.as_str()) {
                Ok(mut config) => {
                    config.fill_with_defaults();
                    config
                }
                Err(e) => {
                    panic!("ERROR: Failed to parse Fankor.toml: {}", e);
                }
            },
            Err(_) => {
                println!("WARNING: Fankor.toml is not present. Using default configuration.");
                FankorConfig::default()
            }
        };

        // Wait enough time to let all other actions to be registered.
        thread::sleep(Duration::from_millis(config.initial_delay.unwrap()));

        let mut idl_build_context = self.build_context();

        // Execute the actions.
        while let Some(action) = self.pop_action() {
            // Executes the function and captures any error to report it to others.
            let arg: &mut IdlBuildContext = &mut idl_build_context;
            let mut arg = AssertUnwindSafe(arg);
            let result = ::std::panic::catch_unwind(move || (action.function)(*arg));

            if let Err(err) = result {
                idl_build_context
                    .set_error_file(format!("{}({})", action.file_path, action.test_name));
                resume_unwind(err);
            }
        }

        println!("All actions done...");

        // Finish the development process.
        self.finish();

        // TODO generate the IDL files.
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// An action to build the IDL.
pub struct IdlAction {
    pub test_name: &'static str,
    pub file_path: &'static str,
    pub function: Box<dyn FnOnce(&mut IdlBuildContext) -> () + UnwindSafe + Send>,
}
