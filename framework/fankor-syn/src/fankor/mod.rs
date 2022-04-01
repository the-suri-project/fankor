pub use config::*;
mod config;

/// Reads the `Fankor.toml` file.
pub fn read_fankor_toml() -> FankorConfig {
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
        Err(e) => {
            panic!("ERROR: missing Fankor.toml: {}", e);
        }
    };

    config.validate();

    config
}
