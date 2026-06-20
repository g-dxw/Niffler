use http::Uri;

use crate::control::{
    audit_admin_read_only_management_token_permissions, management_token_required_permission,
    read_only_management_token_permissions,
};

use super::{classify_control_route, headers};

#[test]
fn classifies_content_moderation_evidence_view_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/content-moderation/evidence/cme-123"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("content_moderation_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("view_evidence"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:content_moderation_evidence")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &decision).as_deref(),
        Some("admin:content_moderation_evidence:read")
    );
    assert!(!read_only_management_token_permissions()
        .iter()
        .any(|permission| permission == "admin:content_moderation_evidence:read"));
    assert!(audit_admin_read_only_management_token_permissions()
        .iter()
        .any(|permission| permission == "admin:content_moderation_evidence:read"));
    assert!(!decision.is_execution_runtime_candidate());
}
