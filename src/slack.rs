use crate::config::Config;
use anyhow::{Context, Result};
use serde::Deserialize;

pub struct Slack {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Deserialize)]
pub struct SlackProfile {
    ok: bool,
    pub profile: Profile,
}

#[derive(Deserialize)]
pub struct SlackPresence {
    ok: bool,
    presence: String,
    online: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Profile {
    pub status_text: String,
    pub status_emoji: String,
}

pub const LISTENING_TO_EMOJI: &str = ":musical_note:";

impl Slack {
    pub fn new(config: &Config) -> Self {
        Slack {
            client: reqwest::Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        "Authorization",
                        format!("Bearer {}", config.slack_token).parse().unwrap(),
                    );
                    headers.insert(
                        "Cookie",
                        format!("d={}", config.slack_cookie).parse().unwrap(),
                    );
                    headers
                })
                .build()
                .unwrap(),
            base_url: config.slack_base_url.clone(),
        }
    }

    pub async fn get_actual_status(self: &Self) -> Result<SlackProfile> {
        let res = self
            .client
            .get(format!("{}/api/users.profile.get", self.base_url))
            .send()
            .await
            .context("Failed to retrieve Slack profile")?;

        let profile = res.json::<SlackProfile>().await?;

        Ok(profile)
    }

    pub async fn get_online_status(self: &Self) -> Result<SlackPresence> {
        let res = self
            .client
            .get("https://slack.com/api/users.getPresence")
            .send()
            .await
            .context("Failed to retrieve Slack presence")?;

        let profile = res.json::<SlackPresence>().await?;

        Ok(profile)
    }

    async fn set_status(self: &Self, status_text: &str, status_emoji: &str) -> Result<()> {
        let payload = serde_json::json!({
            "profile": {
                "status_text": status_text,
                "status_emoji": status_emoji,
                "status_expiration": 0
            }
        });

        self.client
            .post("https://slack.com/api/users.profile.set")
            .json(&payload)
            .send()
            .await
            .context("Failed to set Slack status")?;

        Ok(())
    }

    pub async fn set_listening_to(self: &Self, status_text: &str) -> Result<()> {
        log::info!("Setting Slack status to listening to: {}", status_text);
        self.set_status(status_text, LISTENING_TO_EMOJI).await
    }

    pub async fn clear_status(self: &Self) -> Result<()> {
        log::info!("Clearing Slack status");
        self.set_status("", "").await
    }
}

impl SlackPresence {
    pub fn is_working(&self) -> bool {
        self.ok && self.online && self.presence == "active"
    }
}

impl SlackProfile {
    pub fn is_listening_to(&self) -> bool {
        self.ok && self.profile.status_emoji == LISTENING_TO_EMOJI
    }
}
