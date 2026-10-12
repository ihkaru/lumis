use lumis_core::models::{CursorQuery, EnterpriseRole};

#[test]
fn test_enterprise_role_hierarchy() {
    assert_eq!(EnterpriseRole::SuperAdmin.level(), 5);
    assert_eq!(EnterpriseRole::TenantAdmin.level(), 4);
    assert_eq!(EnterpriseRole::PowerUser.level(), 3);
    assert_eq!(EnterpriseRole::Member.level(), 2);
    assert_eq!(EnterpriseRole::Guest.level(), 1);

    assert!(EnterpriseRole::SuperAdmin > EnterpriseRole::TenantAdmin);
    assert!(EnterpriseRole::TenantAdmin > EnterpriseRole::PowerUser);
    assert!(EnterpriseRole::PowerUser > EnterpriseRole::Member);
    assert!(EnterpriseRole::Member > EnterpriseRole::Guest);
}

#[test]
fn test_enterprise_role_parsing_aliases() {
    assert_eq!(EnterpriseRole::from_str("ciso"), EnterpriseRole::SuperAdmin);
    assert_eq!(EnterpriseRole::from_str("satker_admin"), EnterpriseRole::TenantAdmin);
    assert_eq!(EnterpriseRole::from_str("developer"), EnterpriseRole::PowerUser);
    assert_eq!(EnterpriseRole::from_str("pegawai"), EnterpriseRole::Member);
    assert_eq!(EnterpriseRole::from_str("unknown"), EnterpriseRole::Guest);
}

#[test]
fn test_cursor_query_default() {
    let q = CursorQuery::default();
    assert_eq!(q.limit, 20);
    assert!(q.cursor_time.is_none());
    assert!(q.cursor_id.is_none());
}
