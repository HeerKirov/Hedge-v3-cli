use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct LocalConfigFile {
    debug_mode: Option<bool>,
    application_path: Option<String>,
    server_path: Option<String>,
    userdata_path: Option<String>
}

pub struct LocalConfig {
    debug_mode: bool,
    application_path: Option<String>
}

pub fn load_config() -> LocalConfig {
    let local_config_filepath = "";
    let local_config_text = match fs::read_to_string(local_config_filepath) {
        Ok(data) => data,
        Err(e) => panic!("Cannot load local config {}: {}", local_config_filepath, e)
    };
    let data: LocalConfigFile = match toml::from_str(&*local_config_text) {
        Ok(data) => data,
        Err(e) => panic!("Local config format error: {}", e)
    };

    LocalConfig {
        debug_mode: data.debug_mode.unwrap_or(false),
        application_path: data.application_path
    }
}