use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Test SlackPresence::is_working
#[test]
fn test_is_working_when_ok_online_and_active() {
    let json = r#"{"ok": true, "presence": "active", "online": true}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(presence.is_working());
}

#[test]
fn test_is_working_when_not_ok() {
    let json = r#"{"ok": false, "presence": "active", "online": true}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(!presence.is_working());
}

#[test]
fn test_is_working_when_not_online() {
    let json = r#"{"ok": true, "presence": "active", "online": false}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(!presence.is_working());
}

#[test]
fn test_is_working_when_not_active() {
    let json = r#"{"ok": true, "presence": "away", "online": true}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(!presence.is_working());
}

#[test]
fn test_is_working_when_presence_is_empty() {
    let json = r#"{"ok": true, "presence": "", "online": true}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(!presence.is_working());
}

#[test]
fn test_is_working_all_conditions_false() {
    let json = r#"{"ok": false, "presence": "away", "online": false}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(!presence.is_working());
}

// Test SlackProfile::is_listening_to
#[test]
fn test_is_listening_to_with_musical_note_emoji() {
    let json = r#"{"ok": true, "profile": {"status_text": "Song Name", "status_emoji": ":musical_note:"}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(profile.is_listening_to());
}

#[test]
fn test_is_listening_to_when_not_ok() {
    let json = r#"{"ok": false, "profile": {"status_text": "Song Name", "status_emoji": ":musical_note:"}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(!profile.is_listening_to());
}

#[test]
fn test_is_listening_to_with_different_emoji() {
    let json = r#"{"ok": true, "profile": {"status_text": "Status", "status_emoji": ":coffee:"}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(!profile.is_listening_to());
}

#[test]
fn test_is_listening_to_with_empty_emoji() {
    let json = r#"{"ok": true, "profile": {"status_text": "", "status_emoji": ""}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(!profile.is_listening_to());
}

// Test SlackProfile::has_status
#[test]
fn test_has_status_with_custom_emoji() {
    let json = r#"{"ok": true, "profile": {"status_text": "In a meeting", "status_emoji": ":calendar:"}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(profile.has_status());
}

#[test]
fn test_has_status_when_not_ok() {
    let json = r#"{"ok": false, "profile": {"status_text": "In a meeting", "status_emoji": ":calendar:"}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(!profile.has_status());
}

#[test]
fn test_has_status_with_empty_emoji() {
    let json = r#"{"ok": true, "profile": {"status_text": "Some text", "status_emoji": ""}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(!profile.has_status());
}

#[test]
fn test_has_status_with_listening_to_emoji() {
    let json = r#"{"ok": true, "profile": {"status_text": "Song Name", "status_emoji": ":musical_note:"}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(!profile.has_status());
}

#[test]
fn test_has_status_all_conditions_false() {
    let json = r#"{"ok": false, "profile": {"status_text": "", "status_emoji": ""}}"#;
    let profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert!(!profile.has_status());
}

#[test]
fn test_has_status_with_various_emojis() {
    let emojis = [":coffee:", ":palm_tree:", ":house:", ":computer:", ":phone:"];
    for emoji in emojis {
        let json = format!(r#"{{"ok": true, "profile": {{"status_text": "Status text", "status_emoji": "{}"}}}}"#, emoji);
        let profile: listening_to::slack::SlackProfile = serde_json::from_str(&json).unwrap();
        assert!(profile.has_status(), "Expected has_status to be true for emoji: {}", emoji);
    }
}

// Profile deserialization tests
#[test]
fn test_profile_deserialization() {
    let json = r#"{"status_text": "Working", "status_emoji": ":computer:"}"#;
    let profile: listening_to::slack::Profile = serde_json::from_str(json).unwrap();
    assert_eq!(profile.status_text, "Working");
    assert_eq!(profile.status_emoji, ":computer:");
}

#[test]
fn test_slack_profile_deserialization() {
    let json = r#"{"ok": true, "profile": {"status_text": "Hello", "status_emoji": ":wave:"}}"#;
    let slack_profile: listening_to::slack::SlackProfile = serde_json::from_str(json).unwrap();
    assert_eq!(slack_profile.profile.status_text, "Hello");
    assert_eq!(slack_profile.profile.status_emoji, ":wave:");
}

#[test]
fn test_slack_presence_deserialization() {
    let json = r#"{"ok": true, "presence": "active", "online": true}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(presence.is_working());
}

#[test]
fn test_slack_presence_deserialization_away() {
    let json = r#"{"ok": true, "presence": "away", "online": false}"#;
    let presence: listening_to::slack::SlackPresence = serde_json::from_str(json).unwrap();
    assert!(!presence.is_working());
}

// LISTENING_TO_EMOJI constant test
#[test]
fn test_listening_to_emoji_constant() {
    assert_eq!(listening_to::slack::LISTENING_TO_EMOJI, ":musical_note:");
}

// Integration tests with wiremock
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

    let config = listening_to::config::Config {
        spotify_client_id: "test".to_string(),
        spotify_client_secret: "test".to_string(),
        spotify_redirect_uri: "http://localhost:3000".to_string(),
        spotify_token_cache_path: std::path::PathBuf::from(".test_token_cache.json"),
        slack_base_url: mock_server.uri(),
        slack_token: "test_token".to_string(),
        slack_cookie: "test_cookie".to_string(),
        cron_schedule: "*/10 * * * * *".to_string(),
    };

    let slack = listening_to::slack::Slack::new(&config);
    let result = slack.get_actual_status().await;
    assert!(result.is_ok());

    let profile = result.unwrap();
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

    let config = listening_to::config::Config {
        spotify_client_id: "test".to_string(),
        spotify_client_secret: "test".to_string(),
        spotify_redirect_uri: "http://localhost:3000".to_string(),
        spotify_token_cache_path: std::path::PathBuf::from(".test_token_cache.json"),
        slack_base_url: mock_server.uri(),
        slack_token: "test_token".to_string(),
        slack_cookie: "test_cookie".to_string(),
        cron_schedule: "*/10 * * * * *".to_string(),
    };

    let slack = listening_to::slack::Slack::new(&config);
    let result = slack.get_actual_status().await;
    assert!(result.is_ok());

    let profile = result.unwrap();
    assert!(profile.is_listening_to());
    assert!(!profile.has_status());
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

    let config = listening_to::config::Config {
        spotify_client_id: "test".to_string(),
        spotify_client_secret: "test".to_string(),
        spotify_redirect_uri: "http://localhost:3000".to_string(),
        spotify_token_cache_path: std::path::PathBuf::from(".test_token_cache.json"),
        slack_base_url: mock_server.uri(),
        slack_token: "test_token".to_string(),
        slack_cookie: "test_cookie".to_string(),
        cron_schedule: "*/10 * * * * *".to_string(),
    };

    let slack = listening_to::slack::Slack::new(&config);
    let result = slack.get_actual_status().await;
    assert!(result.is_ok());

    let profile = result.unwrap();
    assert!(!profile.is_listening_to());
    assert!(!profile.has_status());
}

#[tokio::test]
async fn test_slack_new_creates_client_with_base_url() {
    let config = listening_to::config::Config {
        spotify_client_id: "test".to_string(),
        spotify_client_secret: "test".to_string(),
        spotify_redirect_uri: "http://localhost:3000".to_string(),
        spotify_token_cache_path: std::path::PathBuf::from(".test_token_cache.json"),
        slack_base_url: "https://custom.slack.com".to_string(),
        slack_token: "test_token".to_string(),
        slack_cookie: "test_cookie".to_string(),
        cron_schedule: "*/10 * * * * *".to_string(),
    };

    // Just verify construction doesn't panic
    let _slack = listening_to::slack::Slack::new(&config);
}
