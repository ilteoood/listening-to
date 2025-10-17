mod config;
mod listening_to;
mod scheduler;
mod slack;
mod spotify;

use crate::{config::Config, listening_to::ListeningTo, scheduler::parse_cron_interval};
use anyhow::{Context, Result};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env().expect("Failed to load configuration");
    let listening_to = ListeningTo::new(&config).await?;

    loop {
        let interval =
            parse_cron_interval(&config.cron_schedule).context("Failed to parse cron schedule")?;

        sleep(interval).await;

        listening_to.run_check().await?;
    }
}
