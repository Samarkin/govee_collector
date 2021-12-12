use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::stream::StreamExt;
use log::{debug, trace};
use tokio::sync::RwLock;

use crate::collector::govee_h5075::DeviceData;
use crate::device_database::DeviceDatabase;

mod govee_h5075;

pub struct Collector {
    central: Adapter,
    device_database: Arc<DeviceDatabase>,
    known_devices: RwLock<HashMap<PeripheralId, String>>,
    device_data: RwLock<HashMap<String, DeviceData>>,
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum CollectorError {
    #[error("no adapters found")]
    NoAdaptersFound,
}

impl Collector {
    pub async fn new(device_database: Arc<DeviceDatabase>) -> Result<Collector, Box<dyn Error>> {
        let manager = Manager::new().await?;

        // get the first bluetooth adapter
        let adapters = manager.adapters().await?;
        match adapters.into_iter().nth(0) {
            Some(central) => Ok(Collector {
                central,
                device_database,
                known_devices: RwLock::new(HashMap::new()),
                device_data: RwLock::new(HashMap::new()),
            }),
            None => Result::Err(Box::new(CollectorError::NoAdaptersFound))
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let mut events = self.central.events().await?;
        self.central.start_scan(ScanFilter::default()).await?;

        while let Some(event) = events.next().await {
            trace!("Received event {:?}", event);
            match event {
                CentralEvent::DeviceDiscovered(id) => {
                    debug!("Discovered device {:?}", id);
                    if let Ok(peripheral) = self.central.peripheral(&id).await {
                        if let Some(properties) = peripheral.properties().await? {
                            if let Some(local_name) = properties.local_name {
                                if self.device_database.contains_device(&local_name) {
                                    let mut known_devices = self.known_devices.write().await;
                                    known_devices.insert(id.clone(), local_name.clone());
                                    drop(known_devices);
                                    if let Ok(data) = DeviceData::decode(&properties.manufacturer_data) {
                                        debug!("Received initial data from {}: {:?}", local_name, data);
                                        let mut device_data = self.device_data.write().await;
                                        device_data.insert(local_name, data);
                                    }
                                }
                            }
                        }
                    }
                }
                CentralEvent::ManufacturerDataAdvertisement {
                    id,
                    manufacturer_data,
                } => {
                    let known_devices = self.known_devices.read().await;
                    if let Some(local_name) = known_devices.get(&id) {
                        if let Ok(data) = DeviceData::decode(&manufacturer_data) {
                            debug!("Received data from {}: {:?}", local_name, data);
                            let mut device_data = self.device_data.write().await;
                            device_data.insert(local_name.clone(), data);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn get_latest_device_data(&self, local_name: &String) -> Option<DeviceData> {
        let device_data = self.device_data.read().await;
        match device_data.get(local_name) {
            Some(device_data) => Some(device_data.clone()),
            None => None,
        }
    }
}