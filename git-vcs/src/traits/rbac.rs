use crate::types::User;

pub trait Rbac: Send + Sync {
    // 用户管理
    fn create_user(&self, name: &str, pub_key: &str) -> anyhow::Result<()>;
    fn delete_user(&self, name: &str) -> anyhow::Result<()>;
    fn list_users(&self) -> anyhow::Result<Vec<User>>;

    // 权限管理
    fn grant_read(&self, user: &str, repo: &str) -> anyhow::Result<()>;
    fn grant_write(&self, user: &str, repo: &str) -> anyhow::Result<()>;
    fn grant_admin(&self, user: &str, repo: &str) -> anyhow::Result<()>;
    fn revoke_access(&self, user: &str, repo: &str) -> anyhow::Result<()>;

    // 用户组
    fn create_group(&self, name: &str) -> anyhow::Result<()>;
    fn delete_group(&self, name: &str) -> anyhow::Result<()>;
    fn add_to_group(&self, user: &str, group: &str) -> anyhow::Result<()>;
    fn remove_from_group(&self, user: &str, group: &str) -> anyhow::Result<()>;

    // 仓库管理
    fn create_repo(&self, name: &str, group: Option<&str>) -> anyhow::Result<()>;
    fn delete_repo(&self, name: &str) -> anyhow::Result<()>;
    fn rename_repo(&self, old_name: &str, new_name: &str) -> anyhow::Result<()>;
}
