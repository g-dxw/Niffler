use aether_data::repository::audit::AuditLogListQuery;
use aether_data::DataLayerError;
use aether_data_contracts::repository::niffler_core::{
    NifflerBillingReservationListQuery, NifflerBillingReservationStatus,
    NifflerConsistencyCheckListQuery, NifflerReferralRewardLedgerListQuery,
    NifflerReferralRewardLedgerStatus, UpsertNifflerStabilityObservationRecord,
};
use serde_json::json;
use uuid::Uuid;

use crate::data::GatewayDataState;

const WINDOW_SECONDS: u64 = 24 * 60 * 60;
const MAX_CONSISTENCY_SAMPLE: usize = 100;
const ROLLBACK_DRILL_STATUS_KEY: &str = "niffler_stability_rollback_drill_status";
const ROLLBACK_DRILL_EVIDENCE_KEY: &str = "niffler_stability_rollback_drill_evidence";
const INCIDENT_STATUS_KEY: &str = "niffler_stability_incident_status";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NifflerStabilityObservationSummary {
    pub status: String,
    pub blocker_codes: Vec<String>,
    pub window_start_unix_ms: u64,
    pub window_end_unix_ms: u64,
    pub rollback_drill_status: String,
    pub rollback_drill_evidence_complete: bool,
    pub consistency_checked_count: u64,
    pub consistency_issue_count: u64,
    pub unknown_upstream_count: u64,
    pub legacy_write_call_count: u64,
    pub billing_reservation_exception_count: u64,
    pub referral_exception_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NifflerStabilityObservationInput<'a> {
    pub rollback_drill_status: &'a str,
    pub rollback_drill_evidence_complete: bool,
    pub incident_status: &'a str,
    pub audit_reader_available: bool,
    pub request_candidate_reader_available: bool,
    pub consistency_sample_limit_reached: bool,
    pub consistency_issue_count: u64,
    pub unknown_upstream_count: u64,
    pub legacy_write_call_count: u64,
    pub billing_reservation_exception_count: u64,
    pub referral_exception_count: u64,
}

pub(crate) fn classify_niffler_stability_observation(
    input: NifflerStabilityObservationInput<'_>,
) -> (String, Vec<String>) {
    let mut reset_blockers = Vec::new();
    let mut pending_blockers = Vec::new();

    match input.rollback_drill_status {
        "passed" => {
            if !input.rollback_drill_evidence_complete {
                pending_blockers.push("rollback_drill_evidence_missing".to_string());
            }
        }
        "failed" => reset_blockers.push("rollback_drill_failed".to_string()),
        _ => pending_blockers.push("rollback_drill_not_recorded".to_string()),
    }
    match input.incident_status {
        "none" | "" => {}
        "p0" => reset_blockers.push("p0_incident_recorded".to_string()),
        "p1" => reset_blockers.push("p1_incident_recorded".to_string()),
        _ => pending_blockers.push("incident_status_unknown".to_string()),
    }
    if !input.audit_reader_available {
        pending_blockers.push("legacy_write_audit_unavailable".to_string());
    }
    if !input.request_candidate_reader_available {
        pending_blockers.push("request_candidate_audit_unavailable".to_string());
    }
    if input.consistency_sample_limit_reached {
        pending_blockers.push("consistency_sample_limit_reached".to_string());
    }
    if input.consistency_issue_count > 0 {
        reset_blockers.push("consistency_issue".to_string());
    }
    if input.unknown_upstream_count > 0 {
        reset_blockers.push("unknown_upstream".to_string());
    }
    if input.legacy_write_call_count > 0 {
        reset_blockers.push("legacy_write_call".to_string());
    }
    if input.billing_reservation_exception_count > 0 {
        reset_blockers.push("billing_reservation_exception".to_string());
    }
    if input.referral_exception_count > 0 {
        reset_blockers.push("referral_exception".to_string());
    }

    if !reset_blockers.is_empty() {
        reset_blockers.extend(pending_blockers);
        return ("reset_required".to_string(), reset_blockers);
    }
    if !pending_blockers.is_empty() {
        return ("pending".to_string(), pending_blockers);
    }
    ("pass".to_string(), Vec::new())
}

pub(crate) async fn perform_niffler_stability_observation_once(
    data: &GatewayDataState,
) -> Result<Option<NifflerStabilityObservationSummary>, DataLayerError> {
    if !data.has_niffler_core_reader() || !data.has_niffler_core_writer() {
        return Ok(None);
    }

    let now_unix_secs = super::now_unix_secs();
    let (window_start_unix_secs, window_end_unix_secs) =
        stability_window_for_unix_secs(now_unix_secs);
    let window_start_unix_ms = window_start_unix_secs.saturating_mul(1000);
    let window_end_unix_ms = window_end_unix_secs.saturating_mul(1000);
    let observed_at_unix_ms = now_unix_secs.saturating_mul(1000);
    let rollback_drill_status = normalized_status_config(
        data,
        ROLLBACK_DRILL_STATUS_KEY,
        "not_recorded",
        &["passed", "failed", "not_recorded"],
    )
    .await?;
    let rollback_drill_evidence_complete = read_rollback_drill_evidence_complete(data).await?;
    let incident_status =
        normalized_status_config(data, INCIDENT_STATUS_KEY, "none", &["none", "p0", "p1"]).await?;

    let consistency_page = data
        .list_niffler_consistency_checks(&NifflerConsistencyCheckListQuery {
            offset: 0,
            limit: MAX_CONSISTENCY_SAMPLE,
            ..Default::default()
        })
        .await?;
    let consistency_items_in_window = consistency_page
        .items
        .iter()
        .filter(|item| {
            item.created_at_unix_ms >= window_start_unix_ms
                && item.created_at_unix_ms <= window_end_unix_ms
        })
        .collect::<Vec<_>>();
    let consistency_checked_count = consistency_items_in_window.len() as u64;
    let consistency_issue_count = consistency_items_in_window
        .iter()
        .filter(|item| item.consistency_status != "ok")
        .count() as u64;
    let consistency_sample_limit_reached = consistency_page.items.len() >= MAX_CONSISTENCY_SAMPLE
        && consistency_page
            .items
            .last()
            .is_some_and(|item| item.created_at_unix_ms >= window_start_unix_ms);

    let request_candidate_reader_available = data.has_request_candidate_reader();
    let unknown_upstream_count = if request_candidate_reader_available {
        data.count_attempted_request_candidates_with_unknown_upstream_in_window(
            window_start_unix_ms,
            window_end_unix_ms,
        )
        .await?
    } else {
        0
    };

    let audit_reader_available = data.has_admin_audit_log_reader();
    let legacy_write_call_count = if audit_reader_available {
        data.list_admin_audit_logs(&AuditLogListQuery {
            cutoff_unix_secs: window_start_unix_secs,
            username_pattern: None,
            event_type: Some("niffler_legacy_write_frozen".to_string()),
            limit: 1,
            offset: 0,
        })
        .await?
        .total
    } else {
        0
    };

    let billing_reservation_exception_count = count_billing_reservation_exceptions(
        data,
        now_unix_secs,
        window_start_unix_ms,
        window_end_unix_ms,
    )
    .await?;
    let referral_exception_count = data
        .list_niffler_referral_reward_ledger(&NifflerReferralRewardLedgerListQuery {
            status: Some(NifflerReferralRewardLedgerStatus::Failed),
            updated_at_gte_unix_ms: Some(window_start_unix_ms),
            updated_at_lt_unix_ms: Some(window_end_unix_ms),
            offset: 0,
            limit: 1,
            ..Default::default()
        })
        .await?
        .total as u64;

    let (status, blocker_codes) =
        classify_niffler_stability_observation(NifflerStabilityObservationInput {
            rollback_drill_status: rollback_drill_status.as_str(),
            rollback_drill_evidence_complete,
            incident_status: incident_status.as_str(),
            audit_reader_available,
            request_candidate_reader_available,
            consistency_sample_limit_reached,
            consistency_issue_count,
            unknown_upstream_count,
            legacy_write_call_count,
            billing_reservation_exception_count,
            referral_exception_count,
        });
    let summary = NifflerStabilityObservationSummary {
        status,
        blocker_codes,
        window_start_unix_ms,
        window_end_unix_ms,
        rollback_drill_status,
        rollback_drill_evidence_complete,
        consistency_checked_count,
        consistency_issue_count,
        unknown_upstream_count,
        legacy_write_call_count,
        billing_reservation_exception_count,
        referral_exception_count,
    };

    data.upsert_niffler_stability_observation(UpsertNifflerStabilityObservationRecord {
        id: Uuid::new_v4().to_string(),
        window_start_unix_ms,
        window_end_unix_ms,
        status: summary.status.clone(),
        rollback_drill_status: summary.rollback_drill_status.clone(),
        consistency_checked_count,
        consistency_issue_count,
        unknown_upstream_count,
        legacy_write_call_count,
        billing_reservation_exception_count,
        referral_exception_count,
        blocker_codes: summary.blocker_codes.clone(),
        summary: Some(json!({
            "schema_version": 1,
            "window_start_unix_ms": window_start_unix_ms,
            "window_end_unix_ms": window_end_unix_ms,
            "rollback_drill_evidence_complete": rollback_drill_evidence_complete,
            "incident_status": incident_status,
            "audit_reader_available": audit_reader_available,
            "request_candidate_reader_available": request_candidate_reader_available,
            "consistency_sample_limit_reached": consistency_sample_limit_reached
        })),
        created_at_unix_ms: observed_at_unix_ms,
        updated_at_unix_ms: observed_at_unix_ms,
    })
    .await?;

    Ok(Some(summary))
}

fn stability_window_for_unix_secs(now_unix_secs: u64) -> (u64, u64) {
    let window_start_unix_secs = (now_unix_secs / WINDOW_SECONDS).saturating_mul(WINDOW_SECONDS);
    let window_end_unix_secs = window_start_unix_secs.saturating_add(WINDOW_SECONDS);
    (window_start_unix_secs, window_end_unix_secs)
}

async fn normalized_status_config(
    data: &GatewayDataState,
    key: &str,
    default_value: &str,
    allowed_values: &[&str],
) -> Result<String, DataLayerError> {
    let value = data
        .find_system_config_value(key)
        .await?
        .and_then(|value| value.as_str().map(str::to_ascii_lowercase))
        .unwrap_or_else(|| default_value.to_string());
    if allowed_values.iter().any(|allowed| *allowed == value) {
        Ok(value)
    } else {
        Ok("unknown".to_string())
    }
}

async fn read_rollback_drill_evidence_complete(
    data: &GatewayDataState,
) -> Result<bool, DataLayerError> {
    let evidence = data
        .find_system_config_value(ROLLBACK_DRILL_EVIDENCE_KEY)
        .await?;
    Ok(rollback_drill_evidence_is_complete(evidence.as_ref()))
}

fn rollback_drill_evidence_is_complete(evidence: Option<&serde_json::Value>) -> bool {
    let Some(evidence) = evidence.and_then(|value| value.as_object()) else {
        return false;
    };
    matches!(
        evidence.get("status").and_then(|value| value.as_str()),
        Some("passed")
    ) && evidence_text_present(evidence, "backup_reference")
        && evidence_text_present(evidence, "rollback_image_tag")
        && evidence_text_present(evidence, "drill_summary")
        && evidence
            .get("recorded_at_unix_ms")
            .and_then(|value| value.as_u64())
            .is_some()
}

fn evidence_text_present(evidence: &serde_json::Map<String, serde_json::Value>, key: &str) -> bool {
    evidence
        .get(key)
        .and_then(|value| value.as_str())
        .is_some_and(|value| !value.trim().is_empty())
}

async fn count_billing_reservation_exceptions(
    data: &GatewayDataState,
    now_unix_secs: u64,
    window_start_unix_ms: u64,
    window_end_unix_ms: u64,
) -> Result<u64, DataLayerError> {
    let now_unix_ms = now_unix_secs.saturating_mul(1000);
    let manual_review = data
        .list_niffler_billing_reservations(&NifflerBillingReservationListQuery {
            status: Some(NifflerBillingReservationStatus::ManualReview),
            finalized_at_gte_unix_ms: Some(window_start_unix_ms),
            finalized_at_lt_unix_ms: Some(window_end_unix_ms),
            limit: 1,
            ..Default::default()
        })
        .await?
        .total as u64;
    let expired_active = data
        .list_niffler_billing_reservations(&NifflerBillingReservationListQuery {
            status: Some(NifflerBillingReservationStatus::Active),
            expires_at_gte_unix_ms: Some(window_start_unix_ms),
            expires_at_lte_unix_ms: Some(now_unix_ms),
            expires_at_lt_unix_ms: Some(window_end_unix_ms),
            limit: 1,
            ..Default::default()
        })
        .await?
        .total as u64;
    Ok(manual_review + expired_active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stability_observation_window_is_fixed_for_same_utc_day() {
        let (first_start, first_end) = stability_window_for_unix_secs(86_400 + 123);
        let (second_start, second_end) = stability_window_for_unix_secs(86_400 + 80_000);
        assert_eq!((first_start, first_end), (86_400, 172_800));
        assert_eq!((second_start, second_end), (86_400, 172_800));
    }

    #[test]
    fn stability_observation_passes_when_all_inputs_are_clean() {
        let (status, blockers) =
            classify_niffler_stability_observation(NifflerStabilityObservationInput {
                rollback_drill_status: "passed",
                rollback_drill_evidence_complete: true,
                incident_status: "none",
                audit_reader_available: true,
                request_candidate_reader_available: true,
                consistency_sample_limit_reached: false,
                consistency_issue_count: 0,
                unknown_upstream_count: 0,
                legacy_write_call_count: 0,
                billing_reservation_exception_count: 0,
                referral_exception_count: 0,
            });
        assert_eq!(status, "pass");
        assert!(blockers.is_empty());
    }

    #[test]
    fn stability_observation_resets_on_billing_or_routing_blockers() {
        let (status, blockers) =
            classify_niffler_stability_observation(NifflerStabilityObservationInput {
                rollback_drill_status: "failed",
                rollback_drill_evidence_complete: true,
                incident_status: "p1",
                audit_reader_available: true,
                request_candidate_reader_available: true,
                consistency_sample_limit_reached: false,
                consistency_issue_count: 1,
                unknown_upstream_count: 1,
                legacy_write_call_count: 1,
                billing_reservation_exception_count: 1,
                referral_exception_count: 1,
            });
        assert_eq!(status, "reset_required");
        assert!(blockers.contains(&"rollback_drill_failed".to_string()));
        assert!(blockers.contains(&"p1_incident_recorded".to_string()));
        assert!(blockers.contains(&"consistency_issue".to_string()));
        assert!(blockers.contains(&"unknown_upstream".to_string()));
        assert!(blockers.contains(&"legacy_write_call".to_string()));
        assert!(blockers.contains(&"billing_reservation_exception".to_string()));
        assert!(blockers.contains(&"referral_exception".to_string()));
    }

    #[test]
    fn stability_observation_keeps_pending_blockers_visible_when_reset_is_required() {
        let (status, blockers) =
            classify_niffler_stability_observation(NifflerStabilityObservationInput {
                rollback_drill_status: "not_recorded",
                rollback_drill_evidence_complete: false,
                incident_status: "p0",
                audit_reader_available: false,
                request_candidate_reader_available: false,
                consistency_sample_limit_reached: true,
                consistency_issue_count: 0,
                unknown_upstream_count: 0,
                legacy_write_call_count: 0,
                billing_reservation_exception_count: 0,
                referral_exception_count: 0,
            });
        assert_eq!(status, "reset_required");
        assert!(blockers.contains(&"p0_incident_recorded".to_string()));
        assert!(blockers.contains(&"rollback_drill_not_recorded".to_string()));
        assert!(blockers.contains(&"legacy_write_audit_unavailable".to_string()));
        assert!(blockers.contains(&"request_candidate_audit_unavailable".to_string()));
        assert!(blockers.contains(&"consistency_sample_limit_reached".to_string()));
    }

    #[test]
    fn stability_observation_is_pending_when_evidence_is_missing() {
        let (status, blockers) =
            classify_niffler_stability_observation(NifflerStabilityObservationInput {
                rollback_drill_status: "not_recorded",
                rollback_drill_evidence_complete: false,
                incident_status: "none",
                audit_reader_available: false,
                request_candidate_reader_available: false,
                consistency_sample_limit_reached: true,
                consistency_issue_count: 0,
                unknown_upstream_count: 0,
                legacy_write_call_count: 0,
                billing_reservation_exception_count: 0,
                referral_exception_count: 0,
            });
        assert_eq!(status, "pending");
        assert!(blockers.contains(&"rollback_drill_not_recorded".to_string()));
        assert!(blockers.contains(&"legacy_write_audit_unavailable".to_string()));
        assert!(blockers.contains(&"request_candidate_audit_unavailable".to_string()));
        assert!(blockers.contains(&"consistency_sample_limit_reached".to_string()));
    }

    #[test]
    fn stability_observation_is_pending_when_incident_status_is_unknown() {
        let (status, blockers) =
            classify_niffler_stability_observation(NifflerStabilityObservationInput {
                rollback_drill_status: "passed",
                rollback_drill_evidence_complete: true,
                incident_status: "unexpected",
                audit_reader_available: true,
                request_candidate_reader_available: true,
                consistency_sample_limit_reached: false,
                consistency_issue_count: 0,
                unknown_upstream_count: 0,
                legacy_write_call_count: 0,
                billing_reservation_exception_count: 0,
                referral_exception_count: 0,
            });
        assert_eq!(status, "pending");
        assert_eq!(blockers, vec!["incident_status_unknown"]);
    }

    #[test]
    fn stability_observation_is_pending_when_passed_without_evidence() {
        let (status, blockers) =
            classify_niffler_stability_observation(NifflerStabilityObservationInput {
                rollback_drill_status: "passed",
                rollback_drill_evidence_complete: false,
                incident_status: "none",
                audit_reader_available: true,
                request_candidate_reader_available: true,
                consistency_sample_limit_reached: false,
                consistency_issue_count: 0,
                unknown_upstream_count: 0,
                legacy_write_call_count: 0,
                billing_reservation_exception_count: 0,
                referral_exception_count: 0,
            });
        assert_eq!(status, "pending");
        assert_eq!(blockers, vec!["rollback_drill_evidence_missing"]);
    }
}
