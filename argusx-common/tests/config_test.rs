use argusx_common::config::{DatabaseConfig, LoggingConfig, Settings};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_settings_default() {
    let settings = Settings::default();
    assert_eq!(settings.database.path, "prompt_lab/dev.db");
    assert_eq!(settings.database.busy_timeout_ms, 5000);
    assert_eq!(settings.database.max_connections, 5);
    assert_eq!(settings.logging.level, "info");
    assert!(settings.logging.console);
    assert!(settings.logging.file.is_none());
}

#[test]
fn test_settings_load_from_file() {
    // Clean up any existing env vars first
    std::env::remove_var("ARGUSX_DB_PATH");
    std::env::remove_var("ARGUSX_LOG_LEVEL");

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let config_content = r#"
[database]
path = "test.db"
busy_timeout_ms = 1000
max_connections = 10

[logging]
level = "debug"
console = false
"#;
    fs::write(&config_path, config_content).unwrap();

    let settings = Settings::load(config_path.to_str().unwrap()).unwrap();

    assert_eq!(settings.database.path, "test.db");
    assert_eq!(settings.database.busy_timeout_ms, 1000);
    assert_eq!(settings.database.max_connections, 10);
    assert_eq!(settings.logging.level, "debug");
    assert!(!settings.logging.console);
}

#[test]
fn test_settings_env_overrides() {
    // Clean up any existing env vars first
    std::env::remove_var("ARGUSX_DB_PATH");
    std::env::remove_var("ARGUSX_LOG_LEVEL");

    // Set env vars
    std::env::set_var("ARGUSX_DB_PATH", "env_db.db");
    std::env::set_var("ARGUSX_LOG_LEVEL", "warn");

    // Use Settings::load with non-existent file to apply env overrides
    let settings = Settings::load("nonexistent_config.toml").unwrap();

    assert_eq!(settings.database.path, "env_db.db");
    assert_eq!(settings.logging.level, "warn");

    // Clean up
    std::env::remove_var("ARGUSX_DB_PATH");
    std::env::remove_var("ARGUSX_LOG_LEVEL");
}

#[test]
fn test_database_config_default() {
    let config = DatabaseConfig::default();
    assert_eq!(config.path, "prompt_lab/dev.db");
    assert_eq!(config.busy_timeout_ms, 5000);
    assert_eq!(config.max_connections, 5);
}

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();
    assert_eq!(config.level, "info");
    assert!(config.console);
    assert!(config.file.is_none());
}
