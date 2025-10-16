use anyhow::Result;

use crate::{
    config::Config,
    slack::{Slack, SlackProfile},
    spotify::{CurrentPlayingSong, Spotify},
};

pub struct ListeningTo {
    slack: Slack,
    spotify: Spotify,
}

impl ListeningTo {
    pub fn new(config: &Config) -> Self {
        ListeningTo {
            slack: Slack::new(config),
            spotify: Spotify::new(config),
        }
    }

    async fn handle_is_working(
        &self,
        current_playing_song: CurrentPlayingSong,
        slack_profile: SlackProfile,
    ) -> Result<()> {
        if current_playing_song.format() != slack_profile.profile.status_text {
            self.slack
                .set_listening_to(&current_playing_song.format())
                .await?;
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
        current_playing_song: CurrentPlayingSong,
        slack_profile: SlackProfile,
    ) -> Result<()> {
        let slack_presence = self.slack.get_online_status().await?;

        match slack_presence.is_working() {
            true => {
                self.handle_is_working(current_playing_song, slack_profile)
                    .await
            }
            false => self.handle_not_working(slack_profile).await,
        }
    }

    pub async fn run_check(&self) -> Result<()> {
        let current_playing_song = self.spotify.get_current_playing_song().await?;
        let slack_profile = self.slack.get_actual_status().await?;

        match current_playing_song.is_playing {
            true => {
                self.handle_playing_song(current_playing_song, slack_profile)
                    .await?
            }
            false => self.handle_not_working(slack_profile).await?,
        }

        Ok(())
    }
}
