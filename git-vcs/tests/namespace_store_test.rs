use git_vcs::namespace::{NamespaceStore, SqliteNamespaceStore};
use tempfile::TempDir;

#[test]
fn test_create_org() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("test.db");

    let store = SqliteNamespaceStore::new(&db_path).unwrap();
    let org_id = store.create_org("acme", "/git/acme").unwrap();

    assert!(org_id.0 > 0);
}

#[test]
fn test_create_project() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("test.db");

    let store = SqliteNamespaceStore::new(&db_path).unwrap();
    let org_id = store.create_org("acme", "/git/acme").unwrap();
    let project_id = store
        .create_project(org_id, "myapp", Some("My application"))
        .unwrap();

    assert!(project_id.0 > 0);
}
