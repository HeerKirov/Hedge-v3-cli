use std::fs;
use std::path::PathBuf;
use super::config::LocalConfig;
use super::local_data::{LocalDataManager, LocalData};

pub struct ChannelManager<'l> {
    channel_path: PathBuf,
    local_data_manager: &'l LocalDataManager,
    channel: String
}

impl <'l> ChannelManager<'l> {
    pub fn new(config: &LocalConfig, local_data_manager: &'l LocalDataManager) -> ChannelManager<'l> {
        let channel = local_data_manager.read().using_channel.unwrap_or("default".to_string());
        ChannelManager {
            channel_path: config.appdata_path.join("channel"),
            local_data_manager,
            channel
        }
    }
    pub fn use_channel(&self, channel: &str) {
        self.local_data_manager.write(&LocalData { using_channel: Option::Some(channel.to_string()) });
    }
    pub fn current_channel(&self) -> &str {
        self.channel.as_str()
    }
    pub fn list_channel(&self) -> Vec<String> {
        match fs::read_dir(&self.channel_path) {
            Err(e) => panic!("Cannot read channel dir {}: {}", self.channel_path.to_str().unwrap(), e),
            Ok(d) => {
                d.filter_map(|e| e.ok().map(|f| f.file_name().to_str().unwrap().to_string())).collect()
            }
        }
    }
}