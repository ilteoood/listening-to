use anyhow::{Context, Result};
use chrono::DateTime;
use rspotify::{
    AuthCodeSpotify, Credentials, OAuth,
    model::{Actions, AdditionalType, CurrentlyPlayingContext, CurrentlyPlayingType, PlayableItem},
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

        let spotify = AuthCodeSpotify::with_config(
            creds,
            oauth,
            rspotify::Config {
                token_cached: true,
                cache_path: config.spotify_token_cache_path.clone(),
                ..Default::default()
            },
        );

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
            .context("Failed to retrieve currently playing song")?;

        match currently_playing {
            Some(context) => Ok(context),
            None => Ok(CurrentlyPlayingContext {
                context: None,
                timestamp: DateTime::default(),
                progress: None,
                is_playing: false,
                item: None,
                currently_playing_type: CurrentlyPlayingType::Unknown,
                actions: Actions { disallows: vec![] },
            }),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use rspotify::model::{
        AlbumId, ArtistId, FullTrack, SimplifiedArtist, TrackId, Type,
    };
    use std::collections::HashMap;

    fn create_simplified_artist(name: &str) -> SimplifiedArtist {
        SimplifiedArtist {
            external_urls: HashMap::new(),
            href: None,
            id: Some(ArtistId::from_id("artist123").unwrap()),
            name: name.to_string(),
        }
    }

    fn create_full_track(name: &str, artists: Vec<SimplifiedArtist>) -> FullTrack {
        FullTrack {
            album: rspotify::model::SimplifiedAlbum {
                album_type: None,
                artists: vec![],
                external_urls: HashMap::new(),
                href: None,
                id: Some(AlbumId::from_id("album123").unwrap()),
                images: vec![],
                name: "Test Album".to_string(),
                release_date: None,
                release_date_precision: None,
                restrictions: None,
            },
            artists,
            disc_number: 1,
            duration: chrono::TimeDelta::seconds(180),
            explicit: false,
            external_ids: HashMap::new(),
            external_urls: HashMap::new(),
            href: None,
            id: Some(TrackId::from_id("track123").unwrap()),
            is_local: false,
            is_playable: Some(true),
            restrictions: None,
            name: name.to_string(),
            preview_url: None,
            track_number: 1,
            r#type: Type::Track,
        }
    }

    fn create_currently_playing_context(item: Option<PlayableItem>, is_playing: bool) -> CurrentlyPlayingContext {
        CurrentlyPlayingContext {
            context: None,
            timestamp: DateTime::default(),
            progress: None,
            is_playing,
            item,
            currently_playing_type: CurrentlyPlayingType::Track,
            actions: Actions { disallows: vec![] },
        }
    }

    // Helper to test format_currently_playing without needing Spotify client
    fn format_track(track: &FullTrack) -> String {
        let artists: Vec<String> = track.artists.iter().map(|a| a.name.clone()).collect();
        format!("{} - {}", track.name, artists.join(", "))
    }

    #[test]
    fn test_format_track_single_artist() {
        let artist = create_simplified_artist("Artist Name");
        let track = create_full_track("Song Title", vec![artist]);
        
        let result = format_track(&track);
        assert_eq!(result, "Song Title - Artist Name");
    }

    #[test]
    fn test_format_track_multiple_artists() {
        let artists = vec![
            create_simplified_artist("Artist One"),
            create_simplified_artist("Artist Two"),
            create_simplified_artist("Artist Three"),
        ];
        let track = create_full_track("Collaboration Song", artists);
        
        let result = format_track(&track);
        assert_eq!(result, "Collaboration Song - Artist One, Artist Two, Artist Three");
    }

    #[test]
    fn test_format_track_empty_artists() {
        let track = create_full_track("Song Without Artists", vec![]);
        
        let result = format_track(&track);
        assert_eq!(result, "Song Without Artists - ");
    }

    #[test]
    fn test_format_track_special_characters() {
        let artist = create_simplified_artist("AC/DC");
        let track = create_full_track("Back in Black (Remastered)", vec![artist]);
        
        let result = format_track(&track);
        assert_eq!(result, "Back in Black (Remastered) - AC/DC");
    }

    #[test]
    fn test_format_track_unicode() {
        let artist = create_simplified_artist("José González");
        let track = create_full_track("Heartbeats (日本語)", vec![artist]);
        
        let result = format_track(&track);
        assert_eq!(result, "Heartbeats (日本語) - José González");
    }

    #[test]
    fn test_currently_playing_context_with_track() {
        let artist = create_simplified_artist("Artist Name");
        let track = create_full_track("Song Title", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);

        assert!(context.is_playing);
        assert!(context.item.is_some());
    }

    #[test]
    fn test_currently_playing_context_no_item() {
        let context = create_currently_playing_context(None, false);

        assert!(!context.is_playing);
        assert!(context.item.is_none());
    }

    #[test]
    fn test_default_currently_playing_context() {
        let context = CurrentlyPlayingContext {
            context: None,
            timestamp: DateTime::default(),
            progress: None,
            is_playing: false,
            item: None,
            currently_playing_type: CurrentlyPlayingType::Unknown,
            actions: Actions { disallows: vec![] },
        };

        assert!(!context.is_playing);
        assert!(context.item.is_none());
        assert!(context.context.is_none());
        assert!(context.progress.is_none());
    }
}
