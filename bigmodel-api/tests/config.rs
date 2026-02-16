#[test]
fn test_config_new() {
    let config = bigmodel_api::Config::new("test-key");
    assert_eq!(config.api_key, "test-key");
    assert_eq!(config.base_url, "https://open.bigmodel.cn");
}

#[test]
fn test_config_with_base_url() {
    let config = bigmodel_api::Config::new("test-key").with_base_url("https://custom.example.com");
    assert_eq!(config.base_url, "https://custom.example.com");
}
