pub mod git2;
pub mod gitolite_client;

pub use git2::Git2VersionControl;
pub use gitolite_client::{GitoliteClient, GitoliteConfig};

pub mod gitolite {
    pub use super::gitolite_client::{GitoliteClient, GitoliteConfig};
}
