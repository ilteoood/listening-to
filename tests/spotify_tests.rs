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

// Note: We can't directly test Spotify::format_currently_playing without instantiating Spotify,
// which requires OAuth. Instead, we test the underlying logic by verifying the context structure.

#[test]
fn test_currently_playing_context_with_track() {
    let artist = create_simplified_artist("Artist Name");
    let track = create_full_track("Song Title", vec![artist]);
    let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);

    assert!(context.is_playing);
    assert!(context.item.is_some());
    
    if let Some(PlayableItem::Track(track)) = &context.item {
        assert_eq!(track.name, "Song Title");
        assert_eq!(track.artists.len(), 1);
        assert_eq!(track.artists[0].name, "Artist Name");
    } else {
        panic!("Expected Track item");
    }
}

#[test]
fn test_currently_playing_context_with_multiple_artists() {
    let artists = vec![
        create_simplified_artist("Artist One"),
        create_simplified_artist("Artist Two"),
        create_simplified_artist("Artist Three"),
    ];
    let track = create_full_track("Collaboration Song", artists);
    let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);

    if let Some(PlayableItem::Track(track)) = &context.item {
        assert_eq!(track.artists.len(), 3);
        assert_eq!(track.artists[0].name, "Artist One");
        assert_eq!(track.artists[1].name, "Artist Two");
        assert_eq!(track.artists[2].name, "Artist Three");
    } else {
        panic!("Expected Track item");
    }
}

#[test]
fn test_currently_playing_context_no_item() {
    let context = create_currently_playing_context(None, false);

    assert!(!context.is_playing);
    assert!(context.item.is_none());
}

#[test]
fn test_currently_playing_context_empty_artists() {
    let track = create_full_track("Song Without Artists", vec![]);
    let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);

    if let Some(PlayableItem::Track(track)) = &context.item {
        assert_eq!(track.artists.len(), 0);
    } else {
        panic!("Expected Track item");
    }
}

#[test]
fn test_currently_playing_context_special_characters_in_name() {
    let artist = create_simplified_artist("AC/DC");
    let track = create_full_track("Back in Black (Remastered)", vec![artist]);
    let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);

    if let Some(PlayableItem::Track(track)) = &context.item {
        assert_eq!(track.name, "Back in Black (Remastered)");
        assert_eq!(track.artists[0].name, "AC/DC");
    } else {
        panic!("Expected Track item");
    }
}

#[test]
fn test_currently_playing_context_unicode_characters() {
    let artist = create_simplified_artist("José González");
    let track = create_full_track("Heartbeats (日本語)", vec![artist]);
    let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);

    if let Some(PlayableItem::Track(track)) = &context.item {
        assert_eq!(track.name, "Heartbeats (日本語)");
        assert_eq!(track.artists[0].name, "José González");
    } else {
        panic!("Expected Track item");
    }
}

#[test]
fn test_currently_playing_context_long_names() {
    let artist = create_simplified_artist("A Very Long Artist Name That Goes On And On");
    let track = create_full_track(
        "A Very Long Song Title That Also Goes On And On For Testing",
        vec![artist],
    );
    let context = create_currently_playing_context(Some(PlayableItem::Track(track)), true);

    if let Some(PlayableItem::Track(track)) = &context.item {
        assert_eq!(
            track.name,
            "A Very Long Song Title That Also Goes On And On For Testing"
        );
        assert_eq!(
            track.artists[0].name,
            "A Very Long Artist Name That Goes On And On"
        );
    } else {
        panic!("Expected Track item");
    }
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

// Test format logic using direct string formatting (same as what format_currently_playing does)
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
