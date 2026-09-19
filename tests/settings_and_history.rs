use std::fs;

use sublens::{history::HistoryStore, settings::AppSettings};
use tempfile::tempdir;

#[test]
fn settings_round_trip_persists_only_non_secret_controls() {
    assert!(!AppSettings::default().history_enabled);
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let mut settings = AppSettings {
        history_enabled: true,
        timeout_seconds: 24,
        ..AppSettings::default()
    };
    settings.resolver.max_depth = 12;
    settings.resolver.max_discovered_urls = 128;
    settings.save_to_path(&path).unwrap();

    let raw = fs::read_to_string(&path).unwrap();
    assert!(!raw.contains("raw_uri"));
    assert!(!raw.contains("password"));
    let loaded = AppSettings::load_from_path(&path);
    assert!(loaded.history_enabled);
    assert_eq!(loaded.timeout_seconds, 24);
    assert_eq!(loaded.resolver.max_depth, 12);
    assert_eq!(loaded.resolver.max_discovered_urls, 128);
}

#[test]
fn settings_loader_clamps_unsafe_limits() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(
        &path,
        r#"{"timeout_seconds":0,"max_depth":255,"max_discovered_urls":999999}"#,
    )
    .unwrap();

    let loaded = AppSettings::load_from_path(&path);
    assert_eq!(loaded.timeout_seconds, 2);
    assert_eq!(loaded.resolver.max_depth, 16);
    assert_eq!(loaded.resolver.max_discovered_urls, 512);
}

#[test]
fn protected_history_deduplicates_and_clears() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("history.bin");
    let first = "https://first.example/subscription";
    let second = "https://second.example/subscription";
    let mut history = HistoryStore::load_at(path.clone(), true);
    history.append(first);
    history.append(second);
    history.append(first);
    assert_eq!(history.urls(), &[first.to_owned(), second.to_owned()]);
    if let Ok(bytes) = fs::read(&path) {
        let raw = String::from_utf8_lossy(&bytes);
        assert!(!raw.contains(first));
        assert!(!raw.contains(second));
    }

    history.clear();
    assert!(history.urls().is_empty());
    assert!(!path.exists());
}
