use crate::traits::rbac::Rbac;
use crate::types::User;
use anyhow::{anyhow, Context};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum GitoliteConfig {
    Ssh {
        host: String,
        user: String,
        key_path: PathBuf,
    },
    Http {
        base_url: String,
        token: Option<String>,
    },
}

impl GitoliteConfig {
    pub fn ssh(host: &str, user: &str, key_path: impl Into<PathBuf>) -> Self {
        Self::Ssh {
            host: host.to_string(),
            user: user.to_string(),
            key_path: key_path.into(),
        }
    }

    pub fn http(base_url: &str) -> Self {
        Self::Http {
            base_url: base_url.to_string(),
            token: None,
        }
    }
}

pub struct GitoliteClient {
    config: GitoliteConfig,
}

impl GitoliteClient {
    pub fn new(config: GitoliteConfig) -> Self {
        Self { config }
    }

    fn execute_ssh(&self, command: &str) -> anyhow::Result<String> {
        let GitoliteConfig::Ssh {
            host,
            user,
            key_path,
        } = &self.config
        else {
            return Err(anyhow!("SSH config required"));
        };

        let output = Command::new("ssh")
            .args([
                "-i",
                key_path.to_str().unwrap(),
                "-o",
                "StrictHostKeyChecking=no",
                &format!("{}@{}", user, host),
                command,
            ])
            .output()
            .context("Failed to execute SSH command")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn execute_http(&self, command: &str) -> anyhow::Result<String> {
        let GitoliteConfig::Http { base_url, token } = &self.config else {
            return Err(anyhow!("HTTP config required"));
        };

        let client = reqwest::blocking::Client::new();
        let mut request = client.get(format!("{}/{}", base_url, command));

        if let Some(t) = token {
            request = request.header("Authorization", format!("Bearer {}", t));
        }

        let response = request.send().context("HTTP request failed")?;

        if response.status().is_success() {
            Ok(response.text().unwrap_or_default())
        } else {
            Err(anyhow!("HTTP error: {}", response.status()))
        }
    }

    fn execute(&self, command: &str) -> anyhow::Result<String> {
        match &self.config {
            GitoliteConfig::Ssh { .. } => self.execute_ssh(command),
            GitoliteConfig::Http { .. } => self.execute_http(command),
        }
    }
}

impl Rbac for GitoliteClient {
    fn create_user(&self, name: &str, pub_key: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite create-user {} {}", name, pub_key))?;
        Ok(())
    }

    fn delete_user(&self, name: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite delete-user {}", name))?;
        Ok(())
    }

    fn list_users(&self) -> anyhow::Result<Vec<User>> {
        let output = self.execute("gitolite list-users")?;
        let users: Vec<User> = output
            .lines()
            .filter(|line| !line.starts_with("gitolite"))
            .map(|line| User {
                name: line.trim().to_string(),
                pub_key: None,
            })
            .collect();
        Ok(users)
    }

    fn grant_read(&self, user: &str, repo: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite set {} = R {}", user, repo))?;
        Ok(())
    }

    fn grant_write(&self, user: &str, repo: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite set {} = RW {}", user, repo))?;
        Ok(())
    }

    fn grant_admin(&self, user: &str, repo: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite set {} = RW+ {}", user, repo))?;
        Ok(())
    }

    fn revoke_access(&self, user: &str, repo: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite set {} = \"\" {}", user, repo))?;
        Ok(())
    }

    fn create_group(&self, name: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite group {}", name))?;
        Ok(())
    }

    fn delete_group(&self, _name: &str) -> anyhow::Result<()> {
        // Gitolite 没有直接删除组的命令
        Ok(())
    }

    fn add_to_group(&self, user: &str, group: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite set {} @{} = RW {}", user, group, user))?;
        Ok(())
    }

    fn remove_from_group(&self, _user: &str, _group: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn create_repo(&self, name: &str, _group: Option<&str>) -> anyhow::Result<()> {
        self.execute(&format!("gitolite create {}", name))?;
        Ok(())
    }

    fn delete_repo(&self, name: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite delete {}", name))?;
        Ok(())
    }

    fn rename_repo(&self, old_name: &str, new_name: &str) -> anyhow::Result<()> {
        self.execute(&format!("gitolite rename {} {}", old_name, new_name))?;
        Ok(())
    }
}
