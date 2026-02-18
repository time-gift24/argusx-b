use git_vcs::traits::rbac::Rbac;

#[test]
fn test_rbac_trait_exists() {
    fn _check_rbac<R: Rbac>(_rbac: &R) {}
}
