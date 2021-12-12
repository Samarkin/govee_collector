use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use env_logger::Env;
use log::info;
use structopt::StructOpt;

use crate::collector::Collector;
use crate::device_database::DeviceDatabase;
use crate::server::DeviceDataServer;

mod collector;
mod device_database;
mod server;

#[derive(StructOpt)]
#[structopt(name = "govee_collector", about = "Collects data from Govee devices")]
struct Opt {
    #[structopt(short, long, parse(from_os_str), help = "Selects a TOML file with the list of devices")]
    devices_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();
    let device_database = Arc::new(DeviceDatabase::new(opt.devices_file)?);
    let collector = Arc::new(Collector::new(Arc::clone(&device_database)).await?);
    {
        let collector = Arc::clone(&collector);
        tokio::spawn(async move {
            collector.start().await.unwrap();
        });
    }
    let address = "0.0.0.0:50051".parse()?;
    info!("Starting gRPC server at {}", &address);
    DeviceDataServer::serve(device_database, collector, address).await?;
    Ok(())
}