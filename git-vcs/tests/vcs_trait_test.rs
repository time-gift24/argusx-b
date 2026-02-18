use git_vcs::traits::VersionControl;

#[test]
fn test_vcs_trait_exists() {
    // 验证 trait 存在且可被实现
    fn _check_vcs<V: VersionControl>(_vcs: &V) {}
}
