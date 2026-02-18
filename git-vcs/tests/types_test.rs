use git_vcs::types::{Branch, CommitId, User};

#[test]
fn test_commit_id_new_and_access() {
    let id = CommitId::new("abc123");
    assert_eq!(id.as_str(), "abc123");
}

#[test]
fn test_branch_creation() {
    let branch = Branch {
        name: "main".to_string(),
        is_head: true,
        target: CommitId::new("def456"),
    };
    assert_eq!(branch.name, "main");
    assert!(branch.is_head);
}

#[test]
fn test_user_creation() {
    let user = User {
        name: "alice".to_string(),
        pub_key: Some("ssh-rsa AAAA...".to_string()),
    };
    assert_eq!(user.name, "alice");
    assert!(user.pub_key.is_some());
}
