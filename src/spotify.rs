use anyhow::{Context, Result};
use rspotify::{
    AuthCodeSpotify, Credentials, OAuth,
    model::{CurrentlyPlayingContext, PlayableItem},
    prelude::OAuthClient,
    scopes,
};

use crate::config::Config;

pub struct Spotify {
    client: AuthCodeSpotify,
}

impl Spotify {
    pub async fn new(config: &Config) -> Result<Self> {
        let creds = Credentials::new(&config.spotify_client_id, &config.spotify_client_secret);

        let scopes = scopes!("user-read-recently-played");

        let oauth = OAuth {
            redirect_uri: String::from("http://127.0.0.1:3000"),
            scopes,
            ..Default::default()
        };

        let spotify = AuthCodeSpotify::new(creds, oauth);

        let url = spotify.get_authorize_url(false).unwrap();
        spotify.prompt_for_token(&url).await.unwrap();

        Ok(Spotify { client: spotify })
    }

    pub async fn get_currently_playing_song(&self) -> Result<CurrentlyPlayingContext> {
        let currently_playing = self
            .client
            .current_playing(None, Some(vec![]))
            .await
            .context("Failed to retrieve currently playing song")?
            .unwrap();

        Ok(currently_playing)
    }

    pub fn format_currently_playing(&self, currently_playing: &CurrentlyPlayingContext) -> String {
        match &currently_playing.item {
            Some(PlayableItem::Track(item)) => {
                let artists: Vec<String> = item
                    .artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect();
                format!("{} - {}", item.name, artists.join(", "))
            }
            _ => String::new(),
        }
    }
}
