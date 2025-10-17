use anyhow::{Context, Result};
use rspotify::{
    AuthCodeSpotify, Credentials, OAuth,
    model::{AdditionalType, CurrentlyPlayingContext, PlayableItem},
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

        let scopes = scopes!("user-read-currently-playing");

        let oauth = OAuth {
            redirect_uri: config.spotify_redirect_uri.clone(),
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
            .current_playing(
                None,
                Some(vec![&AdditionalType::Track, &AdditionalType::Episode]),
            )
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
