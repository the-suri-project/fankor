use sha2::{Digest, Sha256};
use std::io::{Read, Write};

/// Writes an x into the tasks file to let know the builder the number of tasks to wait for.
pub fn string_to_hash(text: &str) -> String {
    // Compute the hash from the key.
    let mut hasher = Sha256::default();
    hasher.update(text);

    bs58::encode(hasher.finalize()).into_string()
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Writes an x into the tasks file to let know the builder the number of tasks to wait for.
pub fn write_task_hash_to_file(hash: &str) {
    // Create folder.
    let folder_path = std::fs::canonicalize("./target/fankor-build")
        .expect("Cannot canonicalize the path to fankor-build folder");
    std::fs::create_dir_all(&folder_path).unwrap_or_else(|_| {
        panic!(
            "Cannot create the directory at: {}",
            folder_path.to_string_lossy()
        )
    });

    // Open file.
    let file_path = folder_path.join("tasks");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .append(true)
        .open(&file_path)
        .unwrap_or_else(|_| panic!("Cannot open file at: {}", file_path.to_string_lossy()));

    // Read file and check whether the hash is already there.
    let mut content = String::new();
    file.read_to_string(&mut content)
        .unwrap_or_else(|_| panic!("Cannot read file at: {}", file_path.to_string_lossy()));

    if !content.contains(&hash) {
        // Write file.
        file.write_all(format!("{}\n", hash).as_bytes())
            .unwrap_or_else(|_| panic!("Cannot write file at: {}", file_path.to_string_lossy()));
    }
}
