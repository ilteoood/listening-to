mod config;
mod listening_to;
mod slack;
mod spotify;

use std::sync::Arc;

use crate::{config::Config, listening_to::ListeningTo};
use anyhow::Result;
use chrono::Local;
use cron_tab::AsyncCron;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let config = Config::from_env().expect("Failed to load configuration");
    let listening_to = Arc::new(ListeningTo::new(&config).await?);

    let local_tz = Local::now().offset().to_owned();
    let mut cron = AsyncCron::new(local_tz);

    let listening_to = Arc::clone(&listening_to);
    cron.add_fn(&config.cron_schedule, move || {
        let listening_to = Arc::clone(&listening_to);
        async move {
            match listening_to.run_check().await {
                Ok(_) => log::info!("Scheduled check completed successfully"),
                Err(e) => {
                    log::error!("Error during scheduled check: {:?}", e);
                    println!("Error during scheduled check: {:?}", e);
                }
            }
        }
    })
    .await?;

    cron.start_blocking().await;

    Ok(())
}
