use ricercar_control::{
    admit_evidence, assemble_explanation_bundle, govern_admission, route_work_item,
    AdmissionEnvelope, BackendAdmissibility, BackendCanonicalizationPosture,
    BackendMemoryLayoutPosture, BackendParityOracle, BackendRole, BackendRuntimePostureSummary,
    CacheCoherencePosture, CacheLifecycleState, CachePolicySummary, CacheReuseAdmissibility,
    CompatibilityClassification, CompatibilityGateSummary, ComputeEvidenceKind,
    ComputeEvidenceSummary, ComputeSemanticStatus, ComputeValidationPosture, ControlTrace,
    EvidenceProvenance, EvidenceReadiness, ExecutionCommandKind, PluginCompatibility,
    PluginCompatibilityReason, PluginCompatibilitySummary, PrecisionPosture, QueuePriority,
    QueueableWorkItem, ReleaseReadinessSummary, RoutingIntentKind,
};
use ricercar_control_qdiscocirc_adapter::{
    mappers::{map_explanation_bundle, map_governance, map_orchestration_audit_record},
    render_support::stable_probe_summary,
    traces::compare_probe_envelopes,
};

fn hash(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn provenance(artifact_key: &str) -> EvidenceProvenance {
    EvidenceProvenance::new(
        "ricercar-compute",
        "workflow/qdiscocirc",
        artifact_key,
        hash('c'),
        "replay/compute/qdiscocirc-1",
        vec![
            "backend_capability".to_string(),
            "plugin_compatibility".to_string(),
            "cache_recompute_posture".to_string(),
            "compatibility_verdict".to_string(),
            "release_readiness_posture".to_string(),
        ],
    )
}

fn envelope(
    evidence_key: &str,
    evidence_kind: ComputeEvidenceKind,
    summary: ComputeEvidenceSummary,
) -> AdmissionEnvelope {
    AdmissionEnvelope::new(
        evidence_key,
        evidence_kind,
        provenance(evidence_key),
        ComputeValidationPosture::Validated,
        ComputeSemanticStatus::Lawful,
        summary,
    )
}

fn plugin_compatible() -> AdmissionEnvelope {
    envelope(
        "evidence/plugin/compatible",
        ComputeEvidenceKind::PluginCompatibility,
        ComputeEvidenceSummary::PluginCompatibility(PluginCompatibilitySummary {
            plugin_key: "plugin/backend/canonical".to_string(),
            boundary_kind: "backend_capability".to_string(),
            operation_kind: "backend_capability".to_string(),
            compatibility: PluginCompatibility::Compatible,
            reason: PluginCompatibilityReason::DeclaredCompatible,
            backend_admissibility: Some(BackendAdmissibility::Admissible),
        }),
    )
}

fn cache_fresh() -> AdmissionEnvelope {
    envelope(
        "evidence/cache/fresh",
        ComputeEvidenceKind::CachePolicy,
        ComputeEvidenceSummary::CachePolicy(CachePolicySummary {
            lifecycle_state: CacheLifecycleState::Fresh,
            reuse_admissibility: CacheReuseAdmissibility::ReuseAdmissible,
            recompute_reason: None,
            blocked_reason: None,
            coherence_posture: CacheCoherencePosture::Coherent,
        }),
    )
}

fn compatibility_clean() -> AdmissionEnvelope {
    envelope(
        "evidence/compatibility/clean",
        ComputeEvidenceKind::ContractCompatibilityGate,
        ComputeEvidenceSummary::ContractCompatibilityGate(CompatibilityGateSummary {
            classification: CompatibilityClassification::InternalOnly,
            gate_blocking: false,
            version_bump_required: false,
            readiness_bump_required: false,
            reasons: vec!["no_boundary_drift".to_string()],
        }),
    )
}

fn release_ready() -> AdmissionEnvelope {
    release_readiness(
        "evidence/release/ready",
        EvidenceReadiness::Ready,
        "release_ready",
    )
}

fn release_needs_review() -> AdmissionEnvelope {
    release_readiness(
        "evidence/release/needs-review",
        EvidenceReadiness::NeedsReview,
        "cuda_promotion_needs_review",
    )
}

fn release_blocked() -> AdmissionEnvelope {
    release_readiness(
        "evidence/release/blocked",
        EvidenceReadiness::Blocked,
        "memory_readiness_blocked",
    )
}

fn release_readiness(
    evidence_key: &str,
    readiness: EvidenceReadiness,
    reason: &str,
) -> AdmissionEnvelope {
    envelope(
        evidence_key,
        ComputeEvidenceKind::ReleaseReadiness,
        ComputeEvidenceSummary::ReleaseReadiness(ReleaseReadinessSummary {
            readiness,
            reasons: vec![reason.to_string()],
        }),
    )
}

fn backend_admissible() -> AdmissionEnvelope {
    envelope(
        "evidence/backend/admissible",
        ComputeEvidenceKind::BackendAdmissibility,
        ComputeEvidenceSummary::BackendAdmissibility {
            admissibility: BackendAdmissibility::Admissible,
            reason: "backend_admissibility_declared".to_string(),
        },
    )
}

fn backend_runtime_ready() -> AdmissionEnvelope {
    envelope(
        "evidence/backend/runtime-ready",
        ComputeEvidenceKind::BackendRuntimePosture,
        ComputeEvidenceSummary::BackendRuntimePosture(BackendRuntimePostureSummary {
            backend_role: BackendRole::Optimized,
            layout_version: "device_layout_shadow_v1".to_string(),
            layout_posture: BackendMemoryLayoutPosture::DeviceShadowVersioned,
            precision_posture: PrecisionPosture::ExplicitPolicy,
            canonicalization_posture: BackendCanonicalizationPosture::BackendIndependent,
            parity_oracle: BackendParityOracle::CpuReference,
        }),
    )
}

fn work_item(
    key: &str,
    intent: RoutingIntentKind,
    evidence: Vec<AdmissionEnvelope>,
) -> QueueableWorkItem {
    QueueableWorkItem::new(
        key,
        "workflow/qdiscocirc",
        intent,
        QueuePriority::Normal,
        evidence,
    )
    .expect("work item should be valid")
}

#[test]
fn routing_probe_golden_covers_promote_review_and_blocked() {
    let promoted = route_work_item(&work_item(
        "work/qdiscocirc-promote",
        RoutingIntentKind::NormalExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
        ],
    ))
    .expect("promoted route should assemble");
    let review = route_work_item(&work_item(
        "work/qdiscocirc-review",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_needs_review(),
            backend_admissible(),
            backend_runtime_ready(),
        ],
    ))
    .expect("review route should assemble");
    let blocked = route_work_item(&work_item(
        "work/qdiscocirc-blocked",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_blocked(),
            backend_admissible(),
            backend_runtime_ready(),
        ],
    ))
    .expect("blocked route should assemble");

    assert_eq!(
        promoted.command.command_kind,
        ExecutionCommandKind::PromoteForExecution
    );
    assert_eq!(
        review.command.command_kind,
        ExecutionCommandKind::HoldForReview
    );
    assert_eq!(blocked.command.command_kind, ExecutionCommandKind::Escalate);

    let actual = format!(
        "-- promoted --\n{}\n-- review --\n{}\n-- blocked --\n{}\n",
        stable_probe_summary(&map_orchestration_audit_record(&promoted)),
        stable_probe_summary(&map_orchestration_audit_record(&review)),
        stable_probe_summary(&map_orchestration_audit_record(&blocked))
    );

    assert_eq!(
        actual,
        include_str!("goldens/control_routing_probe_v0.golden")
    );
}

#[test]
fn control_probe_diff_highlights_changed_routing_posture() {
    let promoted = route_work_item(&work_item(
        "work/qdiscocirc-diff",
        RoutingIntentKind::NormalExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
        ],
    ))
    .expect("promoted route should assemble");
    let blocked = route_work_item(&work_item(
        "work/qdiscocirc-diff",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_blocked(),
            backend_admissible(),
            backend_runtime_ready(),
        ],
    ))
    .expect("blocked route should assemble");

    let diff = compare_probe_envelopes(
        &map_orchestration_audit_record(&promoted),
        &map_orchestration_audit_record(&blocked),
    );

    assert!(diff
        .explanation
        .changed
        .iter()
        .any(|change| change.source_id.contains("command")));
}

#[test]
fn governance_probe_keeps_positive_reasons_out_of_blocking_reasons() {
    let ready_envelope = plugin_compatible();
    let ready_admission = admit_evidence(&ready_envelope);
    let ready_governance = govern_admission(&ready_envelope, &ready_admission);
    let ready_probe = map_governance(&ready_envelope, &ready_admission, &ready_governance);

    assert!(ready_probe.explanation.blocking_reason_ids.is_empty());

    let review_envelope = release_needs_review();
    let review_admission = admit_evidence(&review_envelope);
    let review_governance = govern_admission(&review_envelope, &review_admission);
    let review_probe = map_governance(&review_envelope, &review_admission, &review_governance);

    assert_eq!(
        review_probe.explanation.blocking_reason_ids,
        vec!["readiness_needs_review".to_string()]
    );
}

#[test]
fn explanation_probe_projects_fragments_surfacing_and_posture_flow() {
    let trace = ControlTrace::new(
        "trace/qdiscocirc-explanation",
        "workflow/qdiscocirc",
        vec![plugin_compatible(), release_needs_review()],
    )
    .expect("trace should validate");
    let bundle = assemble_explanation_bundle(&trace).expect("bundle should assemble");
    let probe = map_explanation_bundle(&bundle);

    assert!(probe
        .nodes
        .iter()
        .any(|node| node.source.source_kind == "explanation_fragment"));
    assert!(probe
        .nodes
        .iter()
        .any(|node| node.source.source_kind == "surfacing_directive"));
    assert!(probe
        .nodes
        .iter()
        .any(|node| node.source.source_kind == "diagram_posture_flow"));
    assert!(probe.traces.iter().any(|trace| trace
        .node_ids
        .iter()
        .any(|node_id| node_id.starts_with("control/posture-flow/"))));
}
