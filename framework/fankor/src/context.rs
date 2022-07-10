use fankor_syn::fankor::{read_fankor_toml, FankorConfig};
use parking_lot::{Mutex, MutexGuard};
use std::collections::HashSet;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe, UnwindSafe};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{fs, thread};

use crate::build::IdlBuildContext;

/// Contains helper data to do the building process.
pub struct IdlContext {
    tasks: Arc<Mutex<IdlTasks>>,
    actions: Arc<Mutex<Vec<IdlAction>>>,
    build_context: Arc<Mutex<IdlBuildContext>>,
}

impl IdlContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new IDL build context.
    pub fn new() -> IdlContext {
        IdlContext {
            tasks: {
                // Read tasks file.
                let file_path = fs::canonicalize("./target/fankor-build/tasks")
                    .expect("Cannot canonicalize the path to fankor-build folder");
                let content = fs::read_to_string(&file_path).unwrap_or_else(|_| {
                    panic!("Cannot read file at: {}", file_path.to_string_lossy())
                });

                let mut tasks = HashSet::new();

                for task_hash in content.split('\n') {
                    if task_hash.is_empty() {
                        continue;
                    }

                    tasks.insert(task_hash.to_string());
                }

                Arc::new(Mutex::new(IdlTasks {
                    task_registration_finished: false,
                    remain_tasks: tasks.clone(),
                    tasks,
                }))
            },
            actions: Arc::new(Mutex::new(Vec::new())),
            build_context: Arc::new(Mutex::new(IdlBuildContext::new())),
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Gets a mutable reference to the IDL build context.
    pub fn build_context(&self) -> MutexGuard<IdlBuildContext> {
        self.build_context.lock()
    }

    /// Gets the remaining number of actions.
    pub fn remaining_actions(&self) -> usize {
        self.actions.lock().len()
    }

    // METHODS ----------------------------------------------------------------

    /// Adds an action to the context.
    pub fn register_action<F>(
        &self,
        task_hash: &'static str,
        test_name: &'static str,
        file_path: &'static str,
        function: F,
    ) where
        F: 'static + FnOnce(&mut IdlBuildContext) + UnwindSafe + Send,
    {
        let mut idl_tasks = self.tasks.lock();

        if idl_tasks.task_registration_finished {
            // Add the task hash to the list of tasks in order to wait for it in the next execution.
            idl_tasks.tasks.insert(task_hash.to_string());
            idl_tasks.save_tasks();

            panic!("Race error: the task has not been registered before starting the building process.");
        }

        if idl_tasks.tasks.contains(task_hash) {
            idl_tasks.remain_tasks.remove(task_hash);
        } else {
            // Add the task hash to the list of tasks in order to wait for it in the next execution.
            idl_tasks.tasks.insert(task_hash.to_string());
        }

        let mut actions = self.actions.lock();
        actions.push(IdlAction {
            task_hash,
            test_name,
            file_path,
            function: Box::new(function),
        });
    }

    /// Pops an action from the context.
    pub fn pop_action(&self) -> Option<IdlAction> {
        let mut actions = self.actions.lock();
        actions.pop()
    }

    /// Performs the build process.
    pub fn build(&self) {
        // Read config.
        let config = read_fankor_toml();

        let start = Instant::now();
        let timeout = config.build().task_wait_timeout.unwrap() as u128;
        loop {
            let mut idl_tasks = self.tasks.lock();

            if idl_tasks.remain_tasks.is_empty() {
                // Finish the task registration.
                idl_tasks.task_registration_finished = true;

                // Save the tasks again.
                idl_tasks.save_tasks();

                break;
            }

            if start.elapsed().as_millis() >= timeout {
                // Remove remaining tasks from the list because they are not present anymore.
                let remain_tasks = std::mem::take(&mut idl_tasks.remain_tasks);
                for task in remain_tasks.iter() {
                    idl_tasks.tasks.remove(task);
                }

                // Finish the task registration.
                idl_tasks.task_registration_finished = true;

                // Save the tasks again.
                idl_tasks.save_tasks();

                break;
            }

            // Wait enough time to let all other actions to be registered.
            thread::sleep(Duration::from_millis(
                config.build().task_wait_interval.unwrap(),
            ));
        }

        println!("All tasks registered");

        println!("Executing actions...");
        let idl_build_context = self.execute_actions(self.build_context());
        println!("Executing actions... [DONE]");

        // Generate the IDL files.
        println!("Generating IDL files...");
        self.generate_idl(&config, idl_build_context);
        println!("Generating IDL files... [DONE]");
    }

    fn execute_actions<'a>(
        &self,
        mut idl_build_context: MutexGuard<'a, IdlBuildContext>,
    ) -> MutexGuard<'a, IdlBuildContext> {
        while let Some(action) = self.pop_action() {
            // Executes the function and captures any error to report it to others.
            let arg = &mut idl_build_context;
            let mut arg = AssertUnwindSafe(arg);
            let result = catch_unwind(move || (action.function)(*arg));

            if let Err(err) = result {
                idl_build_context
                    .set_error_file(format!("{}({})", action.file_path, action.test_name));
                resume_unwind(err);
            }
        }

        idl_build_context
    }

    fn generate_idl(&self, config: &FankorConfig, idl_build_context: MutexGuard<IdlBuildContext>) {
        let folder_path = "target/idl";
        let file_path = format!("{}/{}.json", folder_path, config.program.name);
        let file_path_ts = format!("{}/{}.ts", folder_path, config.program.name);

        // Remove file.
        let _ = fs::remove_file(file_path.as_str());
        let _ = fs::remove_file(file_path_ts.as_str());

        // Create folders.
        fs::create_dir_all(folder_path)
            .unwrap_or_else(|e| panic!("Cannot create folder '{}': {}", folder_path, e));

        // Generate the IDL.
        let idl = idl_build_context.build_idl(config);
        let idl_str = serde_json::to_string(&idl).unwrap();

        // Generate typescript IDL.
        let typescript_idl = format!(
            "export type {} = {};\nexport const IDL = {};",
            config.program.name, idl_str, idl_str
        );

        fs::write(file_path.as_str(), idl_str.as_str())
            .unwrap_or_else(|e| panic!("Cannot write file '{}': {}", file_path, e));

        fs::write(file_path_ts.as_str(), typescript_idl.as_str())
            .unwrap_or_else(|e| panic!("Cannot write file '{}': {}", file_path, e));
    }
}

impl Default for IdlContext {
    fn default() -> Self {
        IdlContext::new()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// An action to build the IDL.
pub struct IdlAction {
    pub task_hash: &'static str,
    pub test_name: &'static str,
    pub file_path: &'static str,
    pub function: Box<dyn FnOnce(&mut IdlBuildContext) + UnwindSafe + Send>,
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// An action to build the IDL.
pub struct IdlTasks {
    pub task_registration_finished: bool,
    pub tasks: HashSet<String>,
    pub remain_tasks: HashSet<String>,
}

impl IdlTasks {
    // METHODS ----------------------------------------------------------------

    pub fn save_tasks(&self) {
        // Create folder.
        let folder_path = fs::canonicalize("./target/fankor-build")
            .expect("Cannot canonicalize the path to fankor-build folder");
        fs::create_dir_all(&folder_path).unwrap_or_else(|_| {
            panic!(
                "Cannot create the directory at: {}",
                folder_path.to_string_lossy()
            )
        });

        // Open file.
        let file_path = folder_path.join("tasks");
        let content = self.tasks.iter().map(|v| v.as_str()).collect::<Vec<_>>();
        let content = content.join("\n");

        fs::write(&file_path, content.as_bytes())
            .unwrap_or_else(|_| panic!("Cannot write file at: {}", file_path.to_string_lossy()));
    }
}
