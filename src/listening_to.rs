use anyhow::Result;
use rspotify::model::CurrentlyPlayingContext;

use crate::{
    config::Config,
    slack::{Slack, SlackProfile},
    spotify::Spotify,
};

pub struct ListeningTo {
    slack: Slack,
    spotify: Spotify,
}

impl ListeningTo {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(ListeningTo {
            slack: Slack::new(config),
            spotify: Spotify::new(config).await?,
        })
    }

    async fn handle_is_working(
        &self,
        currently_playing_song: CurrentlyPlayingContext,
        slack_profile: SlackProfile,
    ) -> Result<()> {
        let formatted_song = self
            .spotify
            .format_currently_playing(&currently_playing_song);
        if formatted_song != slack_profile.profile.status_text {
            self.slack.set_listening_to(&formatted_song).await?;
        }
        Ok(())
    }

    async fn handle_not_working(&self, slack_profile: SlackProfile) -> Result<()> {
        if slack_profile.is_listening_to() {
            self.slack.clear_status().await?;
        }
        Ok(())
    }

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
