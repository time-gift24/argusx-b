// git-vcs/src/lib.rs

pub mod config;
pub mod r#impl;
pub mod namespace;
pub mod traits;
pub mod types;

pub use config::GitUserConfig;
pub use namespace::{
    NamespaceStore, OrgId, Organization, Project, ProjectId, SqliteNamespaceStore,
};
pub use r#impl::{Git2VersionControl, GitoliteClient, GitoliteConfig};
pub use traits::{Rbac, VersionControl};
