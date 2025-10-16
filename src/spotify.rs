use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use serde::Deserialize;

use crate::config::Config;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CurrentPlayingSong {
    pub is_playing: bool,
    item: CurrentPlayingItem,
}

#[derive(Deserialize)]
struct CurrentPlayingItem {
    artists: Vec<CurrentPlayingArtist>,
    name: String,
}

#[derive(Deserialize)]
struct CurrentPlayingArtist {
    name: String,
}

pub struct Spotify {
    client: reqwest::Client,
}

impl Spotify {
    pub fn new(config: &Config) -> Self {
        Spotify {
            client: reqwest::Client::builder()
                .default_headers({
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        "Authorization",
                        format!("Bearer {}", config.spotify_token).parse().unwrap(),
                    );
                    headers
                })
                .build()
                .unwrap(),
        }
    }

    pub async fn get_current_playing_song(&self) -> Result<CurrentPlayingSong> {
        let res = self
            .client
            .get("https://api.spotify.com/v1/me/player/currently-playing")
            .send()
            .await
            .context("Failed to retrieve current playing song")?;

        let current_playing_song = res.json::<CurrentPlayingSong>().await?;
        Ok(current_playing_song)
    }
}

impl CurrentPlayingSong {
    pub fn format(&self) -> String {
        let artists: Vec<String> = self
            .item
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect();
        format!("{} - {}", self.item.name, artists.join(", "))
    }
}
