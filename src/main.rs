use std::error::Error;
use std::sync::Arc;

use crate::collector::Collector;
use crate::device_database::DeviceDatabase;
use crate::server::DeviceDataServer;

mod collector;
mod device_database;
mod server;

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
    let address = "0.0.0.0:50051".parse()?;
    DeviceDataServer::serve(device_database, collector, address).await?;
    Ok(())
}