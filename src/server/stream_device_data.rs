use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use futures::Stream;
use tokio::time::sleep;
use tonic::Status;

use crate::collector::Collector;
use crate::device_database::DeviceDatabase;

use super::govee_collector::{
    DeviceData,
    StreamDeviceDataResponse,
};
use super::utils::extract_device_data;

struct SharedState {
    is_working: bool,
    did_prepare_any_data: bool,
    device_data: Option<Vec<DeviceData>>,
    waker: Option<Waker>,
}

pub struct DeviceDataStream {
    refresh_interval: Duration,
    collector: Arc<Collector>,
    device_database: Arc<DeviceDatabase>,
    unique_ids: Arc<Vec<String>>,
    shared_state: Arc<Mutex<SharedState>>,
}

impl DeviceDataStream {
    pub fn new(
        refresh_interval: Duration,
        collector: Arc<Collector>,
        device_database: Arc<DeviceDatabase>,
        unique_ids: Arc<Vec<String>>,
    ) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            is_working: false,
            did_prepare_any_data: false,
            device_data: None,
            waker: None,
        }));
        DeviceDataStream { refresh_interval, collector, device_database, unique_ids, shared_state }
    }
}

impl Drop for DeviceDataStream {
    fn drop(&mut self) {
        println!("Client disconnected");
        self.shared_state.lock().expect("Could not lock mutex").waker = None;
    }
}

impl Stream for DeviceDataStream {
    type Item = Result<StreamDeviceDataResponse, Status>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock().expect("Could not lock mutex");
        shared_state.waker = Some(ctx.waker().clone());
        if !shared_state.is_working {
            shared_state.is_working = true;
            let did_prepare_any_data = shared_state.did_prepare_any_data;
            let refresh_interval = self.refresh_interval;
            let shared_state = Arc::clone( &self.shared_state);
            let collector = Arc::clone(&self.collector);
            let device_database = Arc::clone(&self.device_database);
            let unique_ids = Arc::clone(&self.unique_ids);
            tokio::spawn(async move {
                if did_prepare_any_data {
                    sleep(refresh_interval).await;
                }
                let device_data = extract_device_data(&collector, &device_database, &unique_ids).await;
                let mut shared_state = shared_state.lock().expect("Could not lock mutex");
                shared_state.did_prepare_any_data = true;
                shared_state.device_data = Some(device_data);
                if let Some(waker) = mem::take(&mut shared_state.waker) {
                    waker.wake();
                }
                shared_state.is_working = false;
            });
        }
        match mem::take(&mut shared_state.device_data) {
            Some(device_data) => {
                Poll::Ready(Some(Ok(StreamDeviceDataResponse { devices: device_data })))
            },
            None => Poll::Pending,
        }
    }
}

