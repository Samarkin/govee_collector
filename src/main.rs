use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use collector::Collector;

use crate::device_database::DeviceDatabase;

mod collector;
mod device_database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let device_database = Arc::new(DeviceDatabase::new());
    let collector = Arc::new(Collector::new(Arc::clone(&device_database)).await?);
    {
        let collector = Arc::clone(&collector);
        tokio::spawn(async move {
            collector.start().await.unwrap();
        });
    }
    loop {
        sleep(Duration::from_secs(1)).await;
        for local_name in device_database.get_all_devices() {
            if let Some(device_data) = collector.get_latest_device_data(local_name).await {
                println!("{}: {:.1} ºC ({:.2} ºF), {:.1}% humidity. Battery: {}%",
                         device_database.get_friendly_name(local_name).unwrap(),
                         device_data.temperature_in_c(),
                         device_data.temperature_in_f(),
                         device_data.humidity(),
                         device_data.battery(),
                )
            }
        }
    }
}