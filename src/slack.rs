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

    pub fn has_status(&self) -> bool {
        self.ok
            && self.profile.status_emoji != ""
            && self.profile.status_emoji != LISTENING_TO_EMOJI
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SlackPresence::is_working tests
    #[test]
    fn test_is_working_when_ok_online_and_active() {
        let json = r#"{"ok": true, "presence": "active", "online": true}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(presence.is_working());
    }

    #[test]
    fn test_is_working_when_not_ok() {
        let json = r#"{"ok": false, "presence": "active", "online": true}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_when_not_online() {
        let json = r#"{"ok": true, "presence": "active", "online": false}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_when_not_active() {
        let json = r#"{"ok": true, "presence": "away", "online": true}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_when_presence_is_empty() {
        let json = r#"{"ok": true, "presence": "", "online": true}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_all_conditions_false() {
        let json = r#"{"ok": false, "presence": "away", "online": false}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(!presence.is_working());
    }

    // SlackProfile::is_listening_to tests
    #[test]
    fn test_is_listening_to_with_musical_note_emoji() {
        let json = r#"{"ok": true, "profile": {"status_text": "Song Name", "status_emoji": ":musical_note:"}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(profile.is_listening_to());
    }

    #[test]
    fn test_is_listening_to_when_not_ok() {
        let json = r#"{"ok": false, "profile": {"status_text": "Song Name", "status_emoji": ":musical_note:"}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.is_listening_to());
    }

    #[test]
    fn test_is_listening_to_with_different_emoji() {
        let json = r#"{"ok": true, "profile": {"status_text": "Status", "status_emoji": ":coffee:"}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.is_listening_to());
    }

    #[test]
    fn test_is_listening_to_with_empty_emoji() {
        let json = r#"{"ok": true, "profile": {"status_text": "", "status_emoji": ""}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.is_listening_to());
    }

    // SlackProfile::has_status tests
    #[test]
    fn test_has_status_with_custom_emoji() {
        let json = r#"{"ok": true, "profile": {"status_text": "In a meeting", "status_emoji": ":calendar:"}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(profile.has_status());
    }

    #[test]
    fn test_has_status_when_not_ok() {
        let json = r#"{"ok": false, "profile": {"status_text": "In a meeting", "status_emoji": ":calendar:"}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_with_empty_emoji() {
        let json = r#"{"ok": true, "profile": {"status_text": "Some text", "status_emoji": ""}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_with_listening_to_emoji() {
        let json = r#"{"ok": true, "profile": {"status_text": "Song Name", "status_emoji": ":musical_note:"}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_all_conditions_false() {
        let json = r#"{"ok": false, "profile": {"status_text": "", "status_emoji": ""}}"#;
        let profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_with_various_emojis() {
        let emojis = [":coffee:", ":palm_tree:", ":house:", ":computer:", ":phone:"];
        for emoji in emojis {
            let json = format!(r#"{{"ok": true, "profile": {{"status_text": "Status text", "status_emoji": "{}"}}}}"#, emoji);
            let profile: SlackProfile = serde_json::from_str(&json).unwrap();
            assert!(profile.has_status(), "Expected has_status to be true for emoji: {}", emoji);
        }
    }

    // Profile deserialization tests
    #[test]
    fn test_profile_deserialization() {
        let json = r#"{"status_text": "Working", "status_emoji": ":computer:"}"#;
        let profile: Profile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.status_text, "Working");
        assert_eq!(profile.status_emoji, ":computer:");
    }

    #[test]
    fn test_slack_profile_deserialization() {
        let json = r#"{"ok": true, "profile": {"status_text": "Hello", "status_emoji": ":wave:"}}"#;
        let slack_profile: SlackProfile = serde_json::from_str(json).unwrap();
        assert_eq!(slack_profile.profile.status_text, "Hello");
        assert_eq!(slack_profile.profile.status_emoji, ":wave:");
    }

    #[test]
    fn test_slack_presence_deserialization() {
        let json = r#"{"ok": true, "presence": "active", "online": true}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(presence.is_working());
    }

    #[test]
    fn test_slack_presence_deserialization_away() {
        let json = r#"{"ok": true, "presence": "away", "online": false}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(!presence.is_working());
    }

    // LISTENING_TO_EMOJI constant test
    #[test]
    fn test_listening_to_emoji_constant() {
        assert_eq!(LISTENING_TO_EMOJI, ":musical_note:");
    }
}
