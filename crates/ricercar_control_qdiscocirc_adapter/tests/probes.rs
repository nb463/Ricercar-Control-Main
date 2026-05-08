use ricercar_control::{
    admit_evidence, assemble_explanation_bundle, control_release_readiness_report,
    evaluate_system_release_governance, govern_admission, route_work_item, AdmissionEnvelope,
    BackendAdmissibility, BackendCanonicalizationPosture, BackendLayoutCompatibility,
    BackendLayoutVersion, BackendMemoryLayoutPosture, BackendPackingPolicy, BackendParityOracle,
    BackendPrecisionMode, BackendRole, BackendRuntimePostureSummary, BackendRuntimeTrack,
    CacheCoherencePosture, CacheLifecycleState, CachePolicySummary, CacheReuseAdmissibility,
    CompatibilityClassification, CompatibilityGateSummary, ComputeEvidenceKind,
    ComputeEvidenceSummary, ComputeSemanticStatus, ComputeValidationPosture,
    ControlReleaseReadinessInput, ControlTrace, CudaBackendPromotionSummary,
    CudaCanonicalizationPosture, CudaParityBudget, CudaParityStatus, CudaPromotionPosture,
    CudaPromotionReason, CudaWorkloadEligibility, CudaWorkloadEligibilityReason,
    EvidenceProvenance, EvidenceReadiness, ExecutionCommandKind, GovernanceIncident,
    GovernanceIncidentKind, GovernanceIncidentResponse, GovernancePolicySet,
    HostDeviceTransferSemantics, PluginCompatibility, PluginCompatibilityReason,
    PluginCompatibilitySummary, PolicyCompatibilityPosture, PrecisionPosture, QueuePriority,
    QueueableWorkItem, ReleaseReadinessSummary, RoutingIntentKind, SystemReleaseGovernanceInput,
};
use ricercar_control_qdiscocirc_adapter::{
    mappers::{
        map_explanation_bundle, map_governance, map_orchestration_audit_record,
        map_system_release_governance_record,
    },
    render_support::{
        operator_view_for_probe, stable_delta_summary, stable_operator_view_summary,
        stable_probe_summary,
    },
    traces::{compare_probe_envelopes, summarize_probe_delta},
    OperatorProbeQuestion, ProbeDeltaCause,
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
    compatibility_gate(
        "evidence/compatibility/clean",
        CompatibilityClassification::InternalOnly,
        false,
        "no_boundary_drift",
    )
}

fn compatibility_blocked() -> AdmissionEnvelope {
    compatibility_gate(
        "evidence/compatibility/blocked",
        CompatibilityClassification::Breaking,
        true,
        "gate_version_changed",
    )
}

fn compatibility_gate(
    evidence_key: &str,
    classification: CompatibilityClassification,
    gate_blocking: bool,
    reason: &str,
) -> AdmissionEnvelope {
    envelope(
        evidence_key,
        ComputeEvidenceKind::ContractCompatibilityGate,
        ComputeEvidenceSummary::ContractCompatibilityGate(CompatibilityGateSummary {
            classification,
            gate_blocking,
            version_bump_required: gate_blocking,
            readiness_bump_required: gate_blocking,
            reasons: vec![reason.to_string()],
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

fn cuda_promotion(
    evidence_key: &str,
    posture: CudaPromotionPosture,
    reason: CudaPromotionReason,
    parity_status: CudaParityStatus,
) -> AdmissionEnvelope {
    envelope(
        evidence_key,
        ComputeEvidenceKind::CudaBackendPromotion,
        ComputeEvidenceSummary::CudaBackendPromotion(CudaBackendPromotionSummary {
            backend_admissibility: BackendAdmissibility::Admissible,
            runtime_track: BackendRuntimeTrack::CudaOptimized,
            layout_version: BackendLayoutVersion::CudaDeviceTensorV0,
            transfer_semantics: HostDeviceTransferSemantics::HostDeviceRoundTrip,
            precision_mode: BackendPrecisionMode::Float32Deterministic,
            packing_policy: BackendPackingPolicy::DeviceLocalContiguous,
            canonicalization_boundary: "backend_independent_compute_artifact_v1".to_string(),
            layout_compatibility: BackendLayoutCompatibility::Canonicalizable,
            parity_budget: CudaParityBudget::BoundedUnits { max_delta_units: 2 },
            observed_delta_units: if parity_status == CudaParityStatus::ParityClean {
                0
            } else {
                1
            },
            canonicalization: CudaCanonicalizationPosture::Canonicalized,
            workload_eligibility: CudaWorkloadEligibility::Eligible,
            eligibility_reason: CudaWorkloadEligibilityReason::EngineWorkloadEligible,
            parity_status,
            promotion_posture: posture,
            promotion_reason: reason,
        }),
    )
}

fn policy() -> GovernancePolicySet {
    GovernancePolicySet::new(
        "policy/control/qdiscocirc",
        "control-policy-v1",
        PolicyCompatibilityPosture::Compatible,
        Vec::new(),
    )
    .expect("policy should validate")
}

fn control_readiness() -> ricercar_control::ControlReleaseReadinessReport {
    control_release_readiness_report(ControlReleaseReadinessInput {
        readiness_key: "control-readiness/qdiscocirc".to_string(),
        policy_set: Some(policy()),
        audit_requirements_satisfied: true,
        governance_trace_corpus_green: true,
        workflow_transition_guards_present: true,
        upstream_compute_evidence_consumable: true,
        rollback_doctrine_present: true,
        replay_note: "qdiscocirc control readiness fixture".to_string(),
    })
    .expect("control readiness should evaluate")
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

fn release_governance_record(
    key: &str,
    evidence: Vec<AdmissionEnvelope>,
    incident: Option<GovernanceIncident>,
) -> ricercar_control::SystemReleaseGovernanceRecord {
    let audit = route_work_item(&work_item(
        &format!("work/qdiscocirc-{key}"),
        RoutingIntentKind::AcceleratedExecution,
        evidence.clone(),
    ))
    .expect("routing should assemble");
    evaluate_system_release_governance(SystemReleaseGovernanceInput {
        governance_key: format!("governance/qdiscocirc-{key}"),
        control_readiness: control_readiness(),
        compute_evidence: evidence,
        orchestration_audit: audit,
        incident,
        replay_note: "qdiscocirc operator corpus fixture".to_string(),
    })
    .expect("governance should evaluate")
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

#[test]
fn q3_control_operator_corpus_covers_final_postures() {
    let clean = release_governance_record(
        "clean-promotion",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/promote",
                CudaPromotionPosture::Promote,
                CudaPromotionReason::PromotionEligible,
                CudaParityStatus::ParityClean,
            ),
        ],
        None,
    );
    let hold = release_governance_record(
        "review-hold",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_needs_review(),
            cuda_promotion(
                "evidence/cuda/hold",
                CudaPromotionPosture::Hold,
                CudaPromotionReason::LayoutReviewRequired,
                CudaParityStatus::ParityClean,
            ),
        ],
        None,
    );
    let blocked = release_governance_record(
        "compatibility-block",
        vec![
            cache_fresh(),
            compatibility_blocked(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/promote-blocked",
                CudaPromotionPosture::Promote,
                CudaPromotionReason::PromotionEligible,
                CudaParityStatus::ParityClean,
            ),
        ],
        None,
    );
    let degraded = release_governance_record(
        "degraded-operation",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/degrade",
                CudaPromotionPosture::Degrade,
                CudaPromotionReason::ParityWithinBudget,
                CudaParityStatus::ParityWithinBudget,
            ),
        ],
        None,
    );
    let fallback = release_governance_record(
        "fallback-only",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/fallback",
                CudaPromotionPosture::Fallback,
                CudaPromotionReason::ParityOverBudget,
                CudaParityStatus::ParityOverBudget,
            ),
        ],
        None,
    );
    let rollback = release_governance_record(
        "rollback-required",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/promote-rollback",
                CudaPromotionPosture::Promote,
                CudaPromotionReason::PromotionEligible,
                CudaParityStatus::ParityClean,
            ),
        ],
        Some(
            GovernanceIncident::new(
                "incident/qdiscocirc-rollback",
                GovernanceIncidentKind::BackendLayoutIncident,
                GovernanceIncidentResponse::RollbackRequired,
                "operator incident requires rollback",
            )
            .expect("incident should validate"),
        ),
    );

    let actual = format!(
        "-- clean-promotion --\n{}\n-- review-hold --\n{}\n-- compatibility-block --\n{}\n-- degraded-operation --\n{}\n-- fallback-only --\n{}\n-- rollback-required --\n{}\n",
        stable_operator_view_summary(&operator_view_for_probe(
            "control/q3/clean-promotion",
            OperatorProbeQuestion::WhatHappened,
            &map_system_release_governance_record(&clean),
        )),
        stable_operator_view_summary(&operator_view_for_probe(
            "control/q3/review-hold",
            OperatorProbeQuestion::WhyHeld,
            &map_system_release_governance_record(&hold),
        )),
        stable_operator_view_summary(&operator_view_for_probe(
            "control/q3/compatibility-block",
            OperatorProbeQuestion::WhatBlockedPromotion,
            &map_system_release_governance_record(&blocked),
        )),
        stable_operator_view_summary(&operator_view_for_probe(
            "control/q3/degraded-operation",
            OperatorProbeQuestion::WhyDegraded,
            &map_system_release_governance_record(&degraded),
        )),
        stable_operator_view_summary(&operator_view_for_probe(
            "control/q3/fallback-only",
            OperatorProbeQuestion::WhyFallbackOnly,
            &map_system_release_governance_record(&fallback),
        )),
        stable_operator_view_summary(&operator_view_for_probe(
            "control/q3/rollback-required",
            OperatorProbeQuestion::WhyRollbackRequired,
            &map_system_release_governance_record(&rollback),
        ))
    );

    assert_eq!(
        actual,
        include_str!("goldens/control_q3_operator_corpus_v0.golden")
    );
}

#[test]
fn q3_delta_identifies_control_consequence_change_without_recomputing_truth() {
    let promoted = release_governance_record(
        "delta-promoted",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/delta",
                CudaPromotionPosture::Promote,
                CudaPromotionReason::PromotionEligible,
                CudaParityStatus::ParityClean,
            ),
        ],
        None,
    );
    let rollback = release_governance_record(
        "delta-rollback",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/delta",
                CudaPromotionPosture::Promote,
                CudaPromotionReason::PromotionEligible,
                CudaParityStatus::ParityClean,
            ),
        ],
        Some(
            GovernanceIncident::new(
                "incident/qdiscocirc-delta-rollback",
                GovernanceIncidentKind::OperatorIncident,
                GovernanceIncidentResponse::RollbackRequired,
                "operator rollback incident changes Control consequence",
            )
            .expect("incident should validate"),
        ),
    );

    let promoted_probe = map_system_release_governance_record(&promoted);
    let rollback_probe = map_system_release_governance_record(&rollback);
    let delta = summarize_probe_delta(&promoted_probe, &rollback_probe);

    assert_eq!(delta.cause, ProbeDeltaCause::ControlConsequenceChanged);
    assert!(delta.compute_truth_refs.is_empty());
    assert!(!delta.control_consequence_refs.is_empty());
    assert!(stable_delta_summary(&delta).contains("delta_cause|control_consequence_changed"));
}

#[test]
fn q3_operator_view_keeps_compute_truth_and_control_consequence_refs_distinct() {
    let record = release_governance_record(
        "ownership-split",
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion(
                "evidence/cuda/ownership",
                CudaPromotionPosture::Fallback,
                CudaPromotionReason::ParityOverBudget,
                CudaParityStatus::ParityOverBudget,
            ),
        ],
        None,
    );
    let probe = map_system_release_governance_record(&record);
    let view = operator_view_for_probe(
        "control/q3/ownership-split",
        OperatorProbeQuestion::OwnershipSplit,
        &probe,
    );

    assert!(view
        .compute_truth_refs
        .contains(&"evidence/cuda/ownership".to_string()));
    assert!(view
        .control_consequence_refs
        .contains(&"governance/qdiscocirc-ownership-split".to_string()));
}
