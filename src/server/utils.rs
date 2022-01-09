use std::time::{UNIX_EPOCH};

use crate::collector::Collector;
use crate::device_database::DeviceDatabase;

use super::govee_collector::DeviceData;

pub async fn extract_device_data(
    collector: &Collector,
    device_database: &DeviceDatabase,
    unique_ids: &Vec<String>,
) -> Vec<DeviceData> {
    let mut devices = vec![];
    for local_name in unique_ids {
        if let Some(device_data) = collector.get_latest_device_data(local_name).await {
            let friendly_name = device_database.get_friendly_name(local_name).unwrap().clone();
            let last_update_timestamp = device_data.last_update_timestamp()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|d| d.as_millis().try_into().ok());
            devices.push(DeviceData {
                unique_id: local_name.clone(),
                friendly_name,
                temperature_in_c: Some(device_data.temperature_in_c()),
                humidity: Some(device_data.humidity()),
                battery: Some(device_data.battery() as f32),
                last_update_timestamp,
            })
        } else if let Some(friendly_name) = device_database.get_friendly_name(local_name) {
            devices.push(DeviceData {
                unique_id: local_name.clone(),
                friendly_name: friendly_name.clone(),
                temperature_in_c: None,
                humidity: None,
                battery: None,
                last_update_timestamp: None,
            })
        }
    }
    devices
}

pub fn resolve_unique_ids(device_database: &DeviceDatabase, input: Vec<String>) -> Vec<String> {
    match input.is_empty() {
        false => input,
        true => device_database.get_all_devices().into_iter().map(|s| s.clone()).collect(),
    }
}
