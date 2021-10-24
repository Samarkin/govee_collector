use std::collections::HashMap;

struct Device {
    friendly_name: String,
}

pub struct DeviceDatabase {
    local_name_to_device: HashMap<String, Device>,
}

impl DeviceDatabase {
    pub fn new() -> DeviceDatabase {
        let mut devices = HashMap::new();
        // TODO: Read from the database
        DeviceDatabase { local_name_to_device: devices }
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