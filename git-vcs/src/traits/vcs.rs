use crate::types::{Branch, Commit, CommitId, Tag};
use std::path::Path;

pub trait VersionControl: Send + Sync {
    // 仓库操作
    fn create_repo(&self, path: &Path) -> anyhow::Result<()>;
    fn clone_repo(&self, url: &str, path: &Path) -> anyhow::Result<()>;

    // 版本控制
    fn add(&self, files: &[&Path]) -> anyhow::Result<()>;
    fn commit(&self, message: &str) -> anyhow::Result<CommitId>;
    fn push(&self, remote: &str, branch: &str) -> anyhow::Result<()>;
    fn pull(&self, remote: &str, branch: &str) -> anyhow::Result<()>;

    // 分支操作
    fn create_branch(&self, name: &str) -> anyhow::Result<()>;
    fn switch_branch(&self, name: &str) -> anyhow::Result<()>;
    fn delete_branch(&self, name: &str) -> anyhow::Result<()>;
    fn list_branches(&self) -> anyhow::Result<Vec<Branch>>;

    // 历史查询
    fn log(&self, path: Option<&Path>, limit: usize) -> anyhow::Result<Vec<Commit>>;
    fn diff(&self, from: &CommitId, to: &CommitId, path: Option<&Path>) -> anyhow::Result<String>;
    fn show(&self, commit: &CommitId) -> anyhow::Result<String>;
    fn blame(&self, path: &Path) -> anyhow::Result<String>;

    // 高级操作
    fn rollback(&self, commit_id: &CommitId) -> anyhow::Result<()>;
    fn create_tag(&self, name: &str, message: Option<&str>) -> anyhow::Result<()>;
    fn list_tags(&self) -> anyhow::Result<Vec<Tag>>;
}
