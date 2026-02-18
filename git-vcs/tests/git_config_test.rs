use git_vcs::config::GitUserConfig;
use tempfile::TempDir;

#[test]
fn test_git_user_config_save_load() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("git.toml");

    let config = GitUserConfig {
        user_name: "Test User".to_string(),
        user_email: "test@example.com".to_string(),
        ssh_key_path: std::path::PathBuf::from("/home/.ssh/id_rsa"),
        default_branch: "main".to_string(),
        commit_template: None,
        signing_key: None,
    };

    config.save(&config_path).unwrap();
    let loaded = GitUserConfig::load(&config_path).unwrap();

    assert_eq!(loaded.user_name, "Test User");
    assert_eq!(loaded.user_email, "test@example.com");
}
