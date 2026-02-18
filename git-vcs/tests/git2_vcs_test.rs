use git_vcs::traits::vcs::VersionControl;
use git_vcs::Git2VersionControl;
use tempfile::TempDir;

#[test]
fn test_create_and_commit() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path().join("test_repo");

    let vcs = Git2VersionControl::new();
    vcs.create_repo(&repo_path).unwrap();

    // 创建测试文件
    std::fs::write(repo_path.join("test.txt"), "hello").unwrap();

    vcs.add(&[&repo_path.join("test.txt")]).unwrap();
    let commit_id = vcs.commit("Initial commit").unwrap();

    assert_eq!(commit_id.as_str().len(), 40); // SHA-1 length
}

#[test]
fn test_list_branches() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path().join("test_repo");

    let vcs = Git2VersionControl::new();
    vcs.create_repo(&repo_path).unwrap();

    // Create initial commit to have a branch
    std::fs::write(repo_path.join("test.txt"), "hello").unwrap();
    vcs.add(&[&repo_path.join("test.txt")]).unwrap();
    vcs.commit("Initial commit").unwrap();

    let branches = vcs.list_branches().unwrap();
    // Check that there is at least one branch (name could be "master" or "main")
    assert!(!branches.is_empty(), "Expected at least one branch");
}
