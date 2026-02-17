use std::env;

pub struct Config {
    pub api_key: Option<String>,
    #[allow(dead_code)]
    pub model: Option<String>,
}

impl Config {
    pub fn from_env(provider: &str) -> Self {
        let api_key_var = format!("{}_API_KEY", provider.to_uppercase());
        let model_var = format!("{}_MODEL", provider.to_uppercase());

        Self {
            api_key: env::var(&api_key_var).ok(),
            model: env::var(&model_var).ok(),
        }
    }

    pub fn get_api_key(&self, cli_key: Option<&str>) -> Option<String> {
        if let Some(key) = cli_key {
            Some(key.to_string())
        } else {
            self.api_key.clone()
        }
    }
}
