#[macro_use] extern crate log;

use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use env_logger::Env;
use structopt::StructOpt;
use tokio::time::{Duration, sleep};

use crate::collector::Collector;
use crate::device_database::DeviceDatabase;
use crate::server::DeviceDataServer;

mod collector;
mod device_database;
mod server;

#[derive(StructOpt)]
#[structopt(
    name = "govee_collector",
    about = "Collects data from Govee devices",
    version = env!("VERGEN_SEMVER"),
)]
struct Opt {
    #[structopt(short = "f", long, parse(from_os_str), help = "Selects a TOML file with the list of devices")]
    devices_file: Option<PathBuf>,

    #[structopt(short, long, help = "Socket address to listen on", default_value="127.0.0.1:50051")]
    address: SocketAddr,

    #[structopt(short, long, help = "BLE initialization delay (in seconds)", default_value="0")]
    delay: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();
    let device_database = Arc::new(DeviceDatabase::new(opt.devices_file)?);
    sleep(Duration::from_secs(opt.delay as u64)).await;
    let collector = Arc::new(Collector::new(Arc::clone(&device_database)).await?);
    {
        let collector = Arc::clone(&collector);
        tokio::spawn(async move {
            collector.start().await.unwrap();
        });
    }
    info!("Starting gRPC server at {}", &opt.address);
    DeviceDataServer::serve(device_database, collector, opt.address).await?;
    Ok(())
}