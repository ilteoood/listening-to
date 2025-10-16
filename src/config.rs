use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub spotify_token: String,
    pub slack_base_url: String,
    pub slack_token: String,
    pub slack_cookie: String,
    pub cron_schedule: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            spotify_token: env::var("SPOTIFY_TOKEN").context("SPOTIFY_TOKEN not set")?,
            slack_base_url: env::var("SLACK_BASE_URL")
                .unwrap_or_else(|_| "https://slack.com".to_string()),
            slack_token: env::var("SLACK_TOKEN").context("SLACK_TOKEN not set")?,
            slack_cookie: env::var("SLACK_COOKIE").context("SLACK_COOKIE not set")?,
            cron_schedule: env::var("CRON_SCHEDULE")
                .unwrap_or_else(|_| "*/10 * 8-18 * * 1-5".to_string()),
        })
    }
}
