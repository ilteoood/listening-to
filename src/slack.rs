use crate::config::Config;
use anyhow::{Context, Result};
use serde::Deserialize;

pub struct Slack {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Deserialize)]
pub struct SlackProfile {
    pub ok: bool,
    pub profile: Profile,
}

#[derive(Deserialize)]
pub struct SlackPresence {
    pub ok: bool,
    pub presence: String,
    pub online: bool,
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
            .get(format!("{}/api/users.getPresence", self.base_url))
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
            .post(format!("{}/api/users.profile.set", self.base_url))
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

    fn create_profile(status_text: &str, status_emoji: &str) -> Profile {
        Profile {
            status_text: status_text.to_string(),
            status_emoji: status_emoji.to_string(),
        }
    }

    fn create_slack_profile(ok: bool, status_text: &str, status_emoji: &str) -> SlackProfile {
        SlackProfile {
            ok,
            profile: create_profile(status_text, status_emoji),
        }
    }

    fn create_slack_presence(ok: bool, online: bool, presence: &str) -> SlackPresence {
        SlackPresence {
            ok,
            online,
            presence: presence.to_string(),
        }
    }

    // SlackPresence::is_working tests
    #[test]
    fn test_is_working_when_ok_online_and_active() {
        let presence = create_slack_presence(true, true, "active");
        assert!(presence.is_working());
    }

    #[test]
    fn test_is_working_when_not_ok() {
        let presence = create_slack_presence(false, true, "active");
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_when_not_online() {
        let presence = create_slack_presence(true, false, "active");
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_when_not_active() {
        let presence = create_slack_presence(true, true, "away");
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_when_presence_is_empty() {
        let presence = create_slack_presence(true, true, "");
        assert!(!presence.is_working());
    }

    #[test]
    fn test_is_working_all_conditions_false() {
        let presence = create_slack_presence(false, false, "away");
        assert!(!presence.is_working());
    }

    // SlackProfile::is_listening_to tests
    #[test]
    fn test_is_listening_to_with_musical_note_emoji() {
        let profile = create_slack_profile(true, "Song Name", LISTENING_TO_EMOJI);
        assert!(profile.is_listening_to());
    }

    #[test]
    fn test_is_listening_to_when_not_ok() {
        let profile = create_slack_profile(false, "Song Name", LISTENING_TO_EMOJI);
        assert!(!profile.is_listening_to());
    }

    #[test]
    fn test_is_listening_to_with_different_emoji() {
        let profile = create_slack_profile(true, "Status", ":coffee:");
        assert!(!profile.is_listening_to());
    }

    #[test]
    fn test_is_listening_to_with_empty_emoji() {
        let profile = create_slack_profile(true, "", "");
        assert!(!profile.is_listening_to());
    }

    // SlackProfile::has_status tests
    #[test]
    fn test_has_status_with_custom_emoji() {
        let profile = create_slack_profile(true, "In a meeting", ":calendar:");
        assert!(profile.has_status());
    }

    #[test]
    fn test_has_status_when_not_ok() {
        let profile = create_slack_profile(false, "In a meeting", ":calendar:");
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_with_empty_emoji() {
        let profile = create_slack_profile(true, "Some text", "");
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_with_listening_to_emoji() {
        let profile = create_slack_profile(true, "Song Name", LISTENING_TO_EMOJI);
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_all_conditions_false() {
        let profile = create_slack_profile(false, "", "");
        assert!(!profile.has_status());
    }

    #[test]
    fn test_has_status_with_various_emojis() {
        let emojis = [":coffee:", ":palm_tree:", ":house:", ":computer:", ":phone:"];
        for emoji in emojis {
            let profile = create_slack_profile(true, "Status text", emoji);
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
        assert!(slack_profile.ok);
        assert_eq!(slack_profile.profile.status_text, "Hello");
        assert_eq!(slack_profile.profile.status_emoji, ":wave:");
    }

    #[test]
    fn test_slack_presence_deserialization() {
        let json = r#"{"ok": true, "presence": "active", "online": true}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(presence.ok);
        assert_eq!(presence.presence, "active");
        assert!(presence.online);
    }

    #[test]
    fn test_slack_presence_deserialization_away() {
        let json = r#"{"ok": true, "presence": "away", "online": false}"#;
        let presence: SlackPresence = serde_json::from_str(json).unwrap();
        assert!(presence.ok);
        assert_eq!(presence.presence, "away");
        assert!(!presence.online);
    }

    // LISTENING_TO_EMOJI constant test
    #[test]
    fn test_listening_to_emoji_constant() {
        assert_eq!(LISTENING_TO_EMOJI, ":musical_note:");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config(base_url: &str) -> Config {
        Config {
            spotify_client_id: "test_spotify_id".to_string(),
            spotify_client_secret: "test_spotify_secret".to_string(),
            spotify_redirect_uri: "http://localhost:3000".to_string(),
            spotify_token_cache_path: PathBuf::from(".test_token_cache.json"),
            slack_base_url: base_url.to_string(),
            slack_token: "test_slack_token".to_string(),
            slack_cookie: "test_slack_cookie".to_string(),
            cron_schedule: "*/10 * * * * *".to_string(),
        }
    }

    #[tokio::test]
    async fn test_slack_new_creates_client_with_base_url() {
        let config = create_test_config("https://custom.slack.com");
        let slack = Slack::new(&config);
        assert_eq!(slack.base_url, "https://custom.slack.com");
    }

    #[tokio::test]
    async fn test_get_actual_status_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/users.profile.get"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true,
                "profile": {
                    "status_text": "Working from home",
                    "status_emoji": ":house:"
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let slack = Slack::new(&config);

        let result = slack.get_actual_status().await;
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert!(profile.ok);
        assert_eq!(profile.profile.status_text, "Working from home");
        assert_eq!(profile.profile.status_emoji, ":house:");
    }

    #[tokio::test]
    async fn test_get_actual_status_with_listening_emoji() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/users.profile.get"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true,
                "profile": {
                    "status_text": "Song - Artist",
                    "status_emoji": ":musical_note:"
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let slack = Slack::new(&config);

        let result = slack.get_actual_status().await;
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert!(profile.is_listening_to());
        assert!(!profile.has_status());
    }

    #[tokio::test]
    async fn test_get_online_status_active() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/users.getPresence"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true,
                "presence": "active",
                "online": true
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let slack = Slack::new(&config);

        let result = slack.get_online_status().await;
        assert!(result.is_ok());

        let presence = result.unwrap();
        assert!(presence.is_working());
    }

    #[tokio::test]
    async fn test_get_online_status_away() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/users.getPresence"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true,
                "presence": "away",
                "online": false
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let slack = Slack::new(&config);

        let result = slack.get_online_status().await;
        assert!(result.is_ok());

        let presence = result.unwrap();
        assert!(!presence.is_working());
    }

    #[tokio::test]
    async fn test_set_listening_to() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/users.profile.set"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let slack = Slack::new(&config);

        let result = slack.set_listening_to("Test Song - Test Artist").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/users.profile.set"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let slack = Slack::new(&config);

        let result = slack.clear_status().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_actual_status_empty_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/users.profile.get"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true,
                "profile": {
                    "status_text": "",
                    "status_emoji": ""
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let slack = Slack::new(&config);

        let result = slack.get_actual_status().await;
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert!(!profile.is_listening_to());
        assert!(!profile.has_status());
    }
}
