use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tonic::{Request, Response, Status};
use tonic::transport::Server;

use govee_collector::{DeviceData, GetDeviceDataRequest, GetDeviceDataResponse};
use govee_collector::device_data_provider_server::{DeviceDataProvider, DeviceDataProviderServer};

use crate::collector::Collector;
use crate::device_database::DeviceDatabase;

mod govee_collector {
    tonic::include_proto!("govee_collector"); // The string specified here must match the proto package name
}

pub struct DeviceDataServer {
    device_database: Arc<DeviceDatabase>,
    collector: Arc<Collector>,
}

impl DeviceDataServer {
    pub async fn serve(
        device_database: Arc<DeviceDatabase>,
        collector: Arc<Collector>,
        address: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        let server = DeviceDataServer { device_database, collector };
        Server::builder()
            .add_service(DeviceDataProviderServer::new(server))
            .serve(address)
            .await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl DeviceDataProvider for DeviceDataServer {
    async fn get_device_data(
        &self,
        request: Request<GetDeviceDataRequest>,
    ) -> Result<Response<GetDeviceDataResponse>, Status> {
        println!("Got a request {:?}", request);
        let mut unique_ids: Vec<String> = request.into_inner().unique_ids;
        if unique_ids.is_empty() {
            for local_name in self.device_database.get_all_devices() {
                unique_ids.push(local_name.clone());
            }
        }
        let mut devices = vec![];
        for local_name in unique_ids {
            if let Some(device_data) = self.collector.get_latest_device_data(&local_name).await {
                let friendly_name = self.device_database.get_friendly_name(&local_name).unwrap().clone();
                devices.push(DeviceData {
                    unique_id: local_name,
                    friendly_name,
                    temperature_in_c: Some(device_data.temperature_in_c()),
                    humidity: Some(device_data.humidity()),
                    battery: Some(device_data.battery() as f32),
                })
            }
        }
        let reply = GetDeviceDataResponse { devices };
        Ok(Response::new(reply))
    }
}