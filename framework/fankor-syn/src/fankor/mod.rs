pub use config::*;
use std::env;
use std::path::PathBuf;

mod config;

/// Reads the `Fankor.toml` file.
pub fn read_fankor_toml() -> FankorConfig {
    // Read config.
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("Could not find `CARGO_MANIFEST_DIR` env variable.");

    let fankor_toml_path = PathBuf::from(manifest_dir).join("Fankor.toml");

    let config = match std::fs::read_to_string(fankor_toml_path.as_path()) {
        Ok(file_content) => match toml::from_str::<FankorConfig>(file_content.as_str()) {
            Ok(mut config) => {
                config.fill_with_defaults();
                config
            }
            Err(e) => {
                panic!(
                    "ERROR: Failed to parse Fankor.toml at {:?}: {}",
                    fankor_toml_path.as_os_str(),
                    e
                );
            }
        },
        Err(e) => {
            panic!(
                "ERROR: missing Fankor.toml at {:?}: {}",
                fankor_toml_path.as_os_str(),
                e
            );
        }
    };

    config.validate();

    config
}
