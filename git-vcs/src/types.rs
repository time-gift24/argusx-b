use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CommitId(pub String);

impl CommitId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: CommitId,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_head: bool,
    pub target: CommitId,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub target: CommitId,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub pub_key: Option<String>,
}
