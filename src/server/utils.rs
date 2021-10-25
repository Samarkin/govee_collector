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
            devices.push(DeviceData {
                unique_id: local_name.clone(),
                friendly_name,
                temperature_in_c: Some(device_data.temperature_in_c()),
                humidity: Some(device_data.humidity()),
                battery: Some(device_data.battery() as f32),
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
