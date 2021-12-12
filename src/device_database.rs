use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use dirs::home_dir;
use log::{error, info};
use serde::Deserialize;
use toml::from_str;

#[derive(Deserialize)]
struct Device {
    friendly_name: String,
}

pub struct DeviceDatabase {
    local_name_to_device: HashMap<String, Device>,
}

impl DeviceDatabase {
    pub fn new(devices_file_path: Option<PathBuf>) -> Result<DeviceDatabase, Box<dyn Error>> {
        let devices = match devices_file_path.or_else(Self::default_devices_file_path) {
            Some(path) => match fs::read_to_string(&path) {
                Ok(file_contents) => from_str(&file_contents)?,
                Err(err) => {
                    error!("ERROR: Unable to read configuration file at {:?}: {:?}", path, err);
                    HashMap::new()
                },
            },
            None => {
                error!("ERROR: Unable to locate configuration file. Please specify its path explicitly");
                HashMap::new()
            }
        };
        info!("Loaded configuration for {} devices", devices.len());
        Ok(DeviceDatabase { local_name_to_device: devices })
    }

    fn default_devices_file_path() -> Option<PathBuf> {
        match home_dir() {
            Some(path) => Some(path.join(".govee_devices.toml")),
            None => None,
        }
    }

    pub fn contains_device(&self, local_name: &String) -> bool {
        self.local_name_to_device.contains_key(local_name)
    }

    pub fn get_friendly_name(&self, local_name: &String) -> Option<&String> {
        match self.local_name_to_device.get(local_name) {
            Some(device) => Some(&device.friendly_name),
            None => None,
        }
    }

    pub fn get_all_devices(&self) -> Vec<&String> {
        self.local_name_to_device.keys().collect()
    }
}