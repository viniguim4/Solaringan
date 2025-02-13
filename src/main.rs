mod config;
mod monitor;
mod trader;
mod event_loops;

use anyhow::Result;
use log::info;
use std::fs::OpenOptions;
use crate::config::Config;
use crate::event_loops::start_monitor_loop;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let logfile = "log.txt";

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(logfile)
        .unwrap();

    let _logger = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Pipe(Box::new(file)))
        .format_timestamp_millis()
        .init();

    info!("Copytrade started!");
    
    // Load configuration
    let config = Config::load()?;

    // Start the monitoring loop
    start_monitor_loop(config.clone(), 0).await?;  // RAYDIUM
    start_monitor_loop(config, 1).await?;  // PUMPFUN

    // Keep the main task running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    Ok(())
}
