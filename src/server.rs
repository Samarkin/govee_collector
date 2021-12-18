use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tonic::{Request, Response, Status};
use tonic::transport::Server;

use govee_collector::{GetDeviceDataRequest, GetDeviceDataResponse, StreamDeviceDataRequest};
use govee_collector::device_data_provider_server::{DeviceDataProvider, DeviceDataProviderServer};
use stream_device_data::DeviceDataStream;
use utils::extract_device_data;
use utils::resolve_unique_ids;

use crate::collector::Collector;
use crate::device_database::DeviceDatabase;

mod stream_device_data;
mod utils;

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
        debug!("Got a request {:?}", request);
        let unique_ids = resolve_unique_ids(&self.device_database, request.into_inner().unique_ids);
        let devices = extract_device_data(&self.collector, &self.device_database, &unique_ids).await;
        let reply = GetDeviceDataResponse { devices };
        Ok(Response::new(reply))
    }

    type StreamDeviceDataStream = Pin<Box<DeviceDataStream>>;

    async fn stream_device_data(
        &self,
        request: Request<StreamDeviceDataRequest>,
    ) -> Result<Response<Self::StreamDeviceDataStream>, Status> {
        debug!("Client connected from: {:?} with request {:?}", request.remote_addr(), request);
        let request = request.into_inner();
        let device_data_stream = Box::pin(DeviceDataStream::new(
            Duration::from_secs(request.refresh_interval_in_secs.unwrap_or(60) as u64),
            Arc::clone(&self.collector),
            Arc::clone(&self.device_database),
            Arc::new(resolve_unique_ids(&self.device_database, request.unique_ids))
        ));
        Ok(Response::new(device_data_stream))
    }
}