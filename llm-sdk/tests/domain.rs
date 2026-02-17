#[test]
fn test_role_variants() {
    use llm_sdk::Role;
    let _ = Role::User;
    let _ = Role::Assistant;
    let _ = Role::Tool;
}
