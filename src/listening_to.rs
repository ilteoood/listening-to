use anyhow::Result;
use rspotify::model::CurrentlyPlayingContext;

use crate::{
    config::Config,
    slack::{Slack, SlackPresence, SlackProfile},
    spotify::{format_currently_playing, Spotify},
};

pub struct ListeningTo {
    slack: Slack,
    spotify: Spotify,
}

/// Determines what action to take when the user is working (online and active)
/// Returns Some(formatted_song) if the status should be updated, None otherwise
pub fn should_update_status(
    currently_playing_song: &CurrentlyPlayingContext,
    slack_profile: &SlackProfile,
) -> Option<String> {
    let formatted_song = format_currently_playing(currently_playing_song);
    if formatted_song != slack_profile.profile.status_text && !slack_profile.has_status() {
        Some(formatted_song)
    } else {
        None
    }
}

/// Determines if we should clear the status when the user is not working
pub fn should_clear_status(slack_profile: &SlackProfile) -> bool {
    slack_profile.is_listening_to()
}

/// Determines the action to take based on the current state
#[derive(Debug, PartialEq)]
pub enum StatusAction {
    UpdateStatus(String),
    ClearStatus,
    NoAction,
}

/// Determines what action to take when a song is playing
pub fn determine_playing_action(
    currently_playing_song: &CurrentlyPlayingContext,
    slack_profile: &SlackProfile,
    slack_presence: &SlackPresence,
) -> StatusAction {
    if slack_presence.is_working() {
        match should_update_status(currently_playing_song, slack_profile) {
            Some(song) => StatusAction::UpdateStatus(song),
            None => StatusAction::NoAction,
        }
    } else if should_clear_status(slack_profile) {
        StatusAction::ClearStatus
    } else {
        StatusAction::NoAction
    }
}

/// Determines what action to take when nothing is playing
pub fn determine_not_playing_action(slack_profile: &SlackProfile) -> StatusAction {
    if should_clear_status(slack_profile) {
        StatusAction::ClearStatus
    } else {
        StatusAction::NoAction
    }
}

impl ListeningTo {
    #[cfg(not(tarpaulin_include))]
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(ListeningTo {
            slack: Slack::new(config),
            spotify: Spotify::new(config).await?,
        })
    }

    #[cfg(not(tarpaulin_include))]
    async fn handle_is_working(
        &self,
        currently_playing_song: CurrentlyPlayingContext,
        slack_profile: SlackProfile,
    ) -> Result<()> {
        if let Some(formatted_song) = should_update_status(&currently_playing_song, &slack_profile)
        {
            self.slack.set_listening_to(&formatted_song).await?;
        }
        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    async fn handle_not_working(&self, slack_profile: SlackProfile) -> Result<()> {
        if should_clear_status(&slack_profile) {
            self.slack.clear_status().await?;
        }
        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    async fn handle_playing_song(
        &self,
        currently_playing_song: CurrentlyPlayingContext,
        slack_profile: SlackProfile,
    ) -> Result<()> {
        let slack_presence = self.slack.get_online_status().await?;

        match slack_presence.is_working() {
            true => {
                self.handle_is_working(currently_playing_song, slack_profile)
                    .await
            }
            false => self.handle_not_working(slack_profile).await,
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub async fn run_check(&self) -> Result<()> {
        let currently_playing_song = self.spotify.get_currently_playing_song().await?;
        let slack_profile = self.slack.get_actual_status().await?;

        match currently_playing_song.is_playing {
            true => {
                self.handle_playing_song(currently_playing_song, slack_profile)
                    .await?
            }
            false => self.handle_not_working(slack_profile).await?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slack::{Profile, LISTENING_TO_EMOJI};
    use chrono::DateTime;
    use rspotify::model::{
        Actions, AlbumId, ArtistId, CurrentlyPlayingContext, CurrentlyPlayingType, FullTrack,
        PlayableItem, SimplifiedArtist, TrackId, Type,
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
                album_group: None,
                artists: vec![],
                available_markets: vec![],
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
            available_markets: vec![],
            disc_number: 1,
            duration: chrono::TimeDelta::seconds(180),
            explicit: false,
            external_ids: HashMap::new(),
            external_urls: HashMap::new(),
            href: None,
            id: Some(TrackId::from_id("track123").unwrap()),
            is_local: false,
            is_playable: Some(true),
            linked_from: None,
            restrictions: None,
            name: name.to_string(),
            popularity: 50,
            preview_url: None,
            track_number: 1,
            r#type: Type::Track,
        }
    }

    fn create_currently_playing_context(
        item: Option<PlayableItem>,
        is_playing: bool,
    ) -> CurrentlyPlayingContext {
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

    fn create_slack_profile(ok: bool, status_text: &str, status_emoji: &str) -> SlackProfile {
        SlackProfile {
            ok,
            profile: Profile {
                status_text: status_text.to_string(),
                status_emoji: status_emoji.to_string(),
            },
        }
    }

    fn create_slack_presence(ok: bool, online: bool, presence: &str) -> SlackPresence {
        SlackPresence {
            ok,
            online,
            presence: presence.to_string(),
        }
    }

    // Tests for should_update_status
    #[test]
    fn test_should_update_status_when_song_differs_and_no_status() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("New Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "Old Song - Artist", "");

        let result = should_update_status(&context, &profile);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "New Song - Artist");
    }

    #[test]
    fn test_should_update_status_when_song_is_same() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "Song - Artist", "");

        let result = should_update_status(&context, &profile);
        assert!(result.is_none());
    }

    #[test]
    fn test_should_update_status_when_user_has_custom_status() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "In a meeting", ":calendar:");

        let result = should_update_status(&context, &profile);
        assert!(result.is_none());
    }

    #[test]
    fn test_should_update_status_when_already_listening() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("New Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "Old Song - Artist", LISTENING_TO_EMOJI);

        let result = should_update_status(&context, &profile);
        assert!(result.is_some());
    }

    // Tests for should_clear_status
    #[test]
    fn test_should_clear_status_when_listening() {
        let profile = create_slack_profile(true, "Song - Artist", LISTENING_TO_EMOJI);
        assert!(should_clear_status(&profile));
    }

    #[test]
    fn test_should_not_clear_status_when_not_listening() {
        let profile = create_slack_profile(true, "In a meeting", ":calendar:");
        assert!(!should_clear_status(&profile));
    }

    #[test]
    fn test_should_not_clear_status_when_no_status() {
        let profile = create_slack_profile(true, "", "");
        assert!(!should_clear_status(&profile));
    }

    // Tests for determine_playing_action
    #[test]
    fn test_determine_playing_action_updates_when_working_and_new_song() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("New Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "", "");
        let presence = create_slack_presence(true, true, "active");

        let result = determine_playing_action(&context, &profile, &presence);
        assert_eq!(result, StatusAction::UpdateStatus("New Song - Artist".to_string()));
    }

    #[test]
    fn test_determine_playing_action_no_action_when_working_and_same_song() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "Song - Artist", "");
        let presence = create_slack_presence(true, true, "active");

        let result = determine_playing_action(&context, &profile, &presence);
        assert_eq!(result, StatusAction::NoAction);
    }

    #[test]
    fn test_determine_playing_action_clears_when_not_working_and_listening() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "Song - Artist", LISTENING_TO_EMOJI);
        let presence = create_slack_presence(true, false, "away");

        let result = determine_playing_action(&context, &profile, &presence);
        assert_eq!(result, StatusAction::ClearStatus);
    }

    #[test]
    fn test_determine_playing_action_no_action_when_not_working_and_custom_status() {
        let artist = create_simplified_artist("Artist");
        let track = create_full_track("Song", vec![artist]);
        let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);
        let profile = create_slack_profile(true, "In a meeting", ":calendar:");
        let presence = create_slack_presence(true, false, "away");

        let result = determine_playing_action(&context, &profile, &presence);
        assert_eq!(result, StatusAction::NoAction);
    }

    // Tests for determine_not_playing_action
    #[test]
    fn test_determine_not_playing_action_clears_when_listening() {
        let profile = create_slack_profile(true, "Song - Artist", LISTENING_TO_EMOJI);

        let result = determine_not_playing_action(&profile);
        assert_eq!(result, StatusAction::ClearStatus);
    }

    #[test]
    fn test_determine_not_playing_action_no_action_when_no_status() {
        let profile = create_slack_profile(true, "", "");

        let result = determine_not_playing_action(&profile);
        assert_eq!(result, StatusAction::NoAction);
    }

    #[test]
    fn test_determine_not_playing_action_no_action_when_custom_status() {
        let profile = create_slack_profile(true, "In a meeting", ":calendar:");

        let result = determine_not_playing_action(&profile);
        assert_eq!(result, StatusAction::NoAction);
    }

    // Test StatusAction enum
    #[test]
    fn test_status_action_debug() {
        let update = StatusAction::UpdateStatus("Song".to_string());
        let clear = StatusAction::ClearStatus;
        let no_action = StatusAction::NoAction;

        assert_eq!(format!("{:?}", update), "UpdateStatus(\"Song\")");
        assert_eq!(format!("{:?}", clear), "ClearStatus");
        assert_eq!(format!("{:?}", no_action), "NoAction");
    }
}
