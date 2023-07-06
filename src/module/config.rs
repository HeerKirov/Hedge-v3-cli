use std::{fs, env, path, io::ErrorKind};
use serde::Deserialize;
use home::home_dir;

#[derive(Deserialize)]
struct LocalConfigFile {
    debug_mode: Option<bool>,
    application_path: Option<String>,
    server_path: Option<String>,
    appdata_path: Option<String>,
    userdata_path: Option<String>
}

pub struct LocalConfig {
    pub debug_mode: bool,
    pub application_path: Option<path::PathBuf>,
    pub userdata_path: path::PathBuf,
    pub server_path: path::PathBuf,
    pub appdata_path: path::PathBuf
}

pub fn load_config() -> LocalConfig {
    let local_config_path = match env::var_os("LOCAL_CONFIG_PATH") {
        Some(p) => p.to_str().unwrap().to_string(),
        None => userdata_path().join("cli/config.toml").to_str().unwrap().to_string()
    };
    let local_config_text = match fs::read_to_string(&local_config_path) {
        Ok(data) => Option::Some(data),
        Err(e) => if e.kind() == ErrorKind::NotFound {
            Option::None
        }else{
            panic!("Cannot load local config {}: {}", &local_config_path, e)
        }
    };
    let data: LocalConfigFile = match local_config_text {
        Some(text) => match toml::from_str(&*text) {
           Ok(data) => data,
           Err(e) => panic!("Local config format error: {}", e)
        }
        None => default_config_file()
    };

    LocalConfig {
        debug_mode: data.debug_mode.unwrap_or(false),
        application_path: data.application_path.map(path::PathBuf::from),
        userdata_path: data.userdata_path.as_ref().map(path::PathBuf::from).unwrap_or_else(|| userdata_path()),
        server_path: data.server_path.as_ref().map(path::PathBuf::from).unwrap_or_else(|| data.userdata_path.as_ref().map(path::PathBuf::from).unwrap_or_else(|| userdata_path()).join("server")),
        appdata_path: data.appdata_path.as_ref().map(path::PathBuf::from).unwrap_or_else(|| data.userdata_path.as_ref().map(path::PathBuf::from).unwrap_or_else(|| userdata_path()).join("appdata")),
    }
}

fn userdata_path() -> std::path::PathBuf {
    match home_dir() {
        Some(u) => {
            if cfg!(target_os = "macos") {
                u.join("Library/Application Support/Hedge-v3")
            }else if cfg!(target_os = "linux") {
                u.join(".config/Hedge-v3")
            }else{
                panic!("Unsupported system platform.")
            }
        },
        None => panic!("Cannot read HOME dir.")
    }
}

fn default_config_file() -> LocalConfigFile {
    LocalConfigFile { 
        debug_mode: Option::None, 
        application_path: Option::None, 
        server_path: Option::None, 
        appdata_path: Option::None, 
        userdata_path: Option::None
    }
}