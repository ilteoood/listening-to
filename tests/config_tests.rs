use std::sync::Mutex;
use std::env;

// Mutex to ensure tests don't run in parallel since they modify env vars
static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn setup_required_env_vars() {
    // SAFETY: Environment variable modification is unsafe in Rust 2024.
    // This is protected by ENV_MUTEX ensuring single-threaded access.
    unsafe {
        env::set_var("SPOTIFY_CLIENT_ID", "test_client_id");
        env::set_var("SPOTIFY_CLIENT_SECRET", "test_client_secret");
        env::set_var("SLACK_TOKEN", "test_slack_token");
        env::set_var("SLACK_COOKIE", "test_slack_cookie");
    }
}

fn clear_env_vars() {
    // SAFETY: Environment variable modification is unsafe in Rust 2024.
    // This is protected by ENV_MUTEX ensuring single-threaded access.
    unsafe {
        env::remove_var("SPOTIFY_CLIENT_ID");
        env::remove_var("SPOTIFY_CLIENT_SECRET");
        env::remove_var("SPOTIFY_REDIRECT_URI");
        env::remove_var("SPOTIFY_TOKEN_CACHE_PATH");
        env::remove_var("SLACK_BASE_URL");
        env::remove_var("SLACK_TOKEN");
        env::remove_var("SLACK_COOKIE");
        env::remove_var("CRON_SCHEDULE");
    }
}

#[test]
fn test_from_env_with_all_required_vars() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    setup_required_env_vars();

    let config = listening_to::config::Config::from_env().unwrap();

    assert_eq!(config.spotify_client_id, "test_client_id");
    assert_eq!(config.spotify_client_secret, "test_client_secret");
    assert_eq!(config.slack_token, "test_slack_token");
    assert_eq!(config.slack_cookie, "test_slack_cookie");
}

#[test]
fn test_from_env_uses_default_values() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    setup_required_env_vars();

    let config = listening_to::config::Config::from_env().unwrap();

    assert_eq!(config.spotify_redirect_uri, "http://127.0.0.1:3000");
    assert_eq!(
        config.spotify_token_cache_path,
        std::path::PathBuf::from(".spotify_token_cache.json")
    );
    assert_eq!(config.slack_base_url, "https://slack.com");
    assert_eq!(config.cron_schedule, "*/10 * 8-18 * * 1-5");
}

#[test]
fn test_from_env_with_custom_optional_vars() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    setup_required_env_vars();
    // SAFETY: Environment variable modification is unsafe in Rust 2024.
    // This is protected by ENV_MUTEX ensuring single-threaded access.
    unsafe {
        env::set_var("SPOTIFY_REDIRECT_URI", "http://custom:8080");
        env::set_var("SPOTIFY_TOKEN_CACHE_PATH", "/custom/path/token.json");
        env::set_var("SLACK_BASE_URL", "https://custom.slack.com");
        env::set_var("CRON_SCHEDULE", "0 * * * * *");
    }

    let config = listening_to::config::Config::from_env().unwrap();

    assert_eq!(config.spotify_redirect_uri, "http://custom:8080");
    assert_eq!(
        config.spotify_token_cache_path,
        std::path::PathBuf::from("/custom/path/token.json")
    );
    assert_eq!(config.slack_base_url, "https://custom.slack.com");
    assert_eq!(config.cron_schedule, "0 * * * * *");
}

#[test]
fn test_from_env_missing_spotify_client_id() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    // SAFETY: Environment variable modification is unsafe in Rust 2024.
    // This is protected by ENV_MUTEX ensuring single-threaded access.
    unsafe {
        env::set_var("SPOTIFY_CLIENT_SECRET", "test_secret");
        env::set_var("SLACK_TOKEN", "test_token");
        env::set_var("SLACK_COOKIE", "test_cookie");
    }

    let result = listening_to::config::Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SPOTIFY_CLIENT_ID not set"));
}

#[test]
fn test_from_env_missing_spotify_client_secret() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    // SAFETY: Environment variable modification is unsafe in Rust 2024.
    // This is protected by ENV_MUTEX ensuring single-threaded access.
    unsafe {
        env::set_var("SPOTIFY_CLIENT_ID", "test_id");
        env::set_var("SLACK_TOKEN", "test_token");
        env::set_var("SLACK_COOKIE", "test_cookie");
    }

    let result = listening_to::config::Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SPOTIFY_CLIENT_SECRET not set"));
}

#[test]
fn test_from_env_missing_slack_token() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    // SAFETY: Environment variable modification is unsafe in Rust 2024.
    // This is protected by ENV_MUTEX ensuring single-threaded access.
    unsafe {
        env::set_var("SPOTIFY_CLIENT_ID", "test_id");
        env::set_var("SPOTIFY_CLIENT_SECRET", "test_secret");
        env::set_var("SLACK_COOKIE", "test_cookie");
    }

    let result = listening_to::config::Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SLACK_TOKEN not set"));
}

#[test]
fn test_from_env_missing_slack_cookie() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    // SAFETY: Environment variable modification is unsafe in Rust 2024.
    // This is protected by ENV_MUTEX ensuring single-threaded access.
    unsafe {
        env::set_var("SPOTIFY_CLIENT_ID", "test_id");
        env::set_var("SPOTIFY_CLIENT_SECRET", "test_secret");
        env::set_var("SLACK_TOKEN", "test_token");
    }

    let result = listening_to::config::Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SLACK_COOKIE not set"));
}

#[test]
fn test_config_clone() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    setup_required_env_vars();

    let config = listening_to::config::Config::from_env().unwrap();
    let cloned = config.clone();

    assert_eq!(config.spotify_client_id, cloned.spotify_client_id);
    assert_eq!(config.spotify_client_secret, cloned.spotify_client_secret);
    assert_eq!(config.spotify_redirect_uri, cloned.spotify_redirect_uri);
    assert_eq!(
        config.spotify_token_cache_path,
        cloned.spotify_token_cache_path
    );
    assert_eq!(config.slack_base_url, cloned.slack_base_url);
    assert_eq!(config.slack_token, cloned.slack_token);
    assert_eq!(config.slack_cookie, cloned.slack_cookie);
    assert_eq!(config.cron_schedule, cloned.cron_schedule);
}

#[test]
fn test_config_debug() {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_env_vars();
    setup_required_env_vars();

    let config = listening_to::config::Config::from_env().unwrap();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("spotify_client_id"));
    assert!(debug_str.contains("test_client_id"));
}
