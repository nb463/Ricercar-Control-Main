use ricercar_control::{
    admit_evidence, route_work_item, AdmissionEnvelope, AdmissionOutcome, BackendAdmissibility,
    BackendCanonicalizationPosture, BackendLayoutCompatibility, BackendLayoutVersion,
    BackendMemoryLayoutPosture, BackendPackingPolicy, BackendParityOracle, BackendPrecisionMode,
    BackendRole, BackendRuntimePostureSummary, BackendRuntimeTrack, CacheBlockedReason,
    CacheCoherencePosture, CacheLifecycleState, CachePolicySummary, CacheReuseAdmissibility,
    CompatibilityClassification, CompatibilityGateSummary, ComputeEvidenceKind,
    ComputeEvidenceSummary, ComputeSemanticStatus, ComputeValidationPosture,
    CudaBackendPromotionSummary, CudaCanonicalizationPosture, CudaParityBudget, CudaParityStatus,
    CudaPromotionPosture, CudaPromotionReason, CudaWorkloadEligibility,
    CudaWorkloadEligibilityReason, EvidenceProvenance, EvidenceReadiness, ExecutionCommandKind,
    HostDeviceTransferSemantics, OrchestrationState, PluginCompatibility,
    PluginCompatibilityReason, PluginCompatibilitySummary, PrecisionPosture, QueuePriority,
    QueueableWorkItem, RecomputeReason, ReleaseReadinessSummary, RoutingIntentKind, RoutingReason,
};

fn hash(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn provenance(artifact_key: &str) -> EvidenceProvenance {
    EvidenceProvenance::new(
        "ricercar-compute",
        "workflow/orchestration",
        artifact_key,
        hash('b'),
        "replay/compute/orchestration-1",
        vec![
            "backend_capability".to_string(),
            "plugin_compatibility".to_string(),
            "cache_recompute_posture".to_string(),
            "compatibility_verdict".to_string(),
            "release_readiness".to_string(),
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

fn cache_stale_recompute() -> AdmissionEnvelope {
    envelope(
        "evidence/cache/stale",
        ComputeEvidenceKind::CachePolicy,
        ComputeEvidenceSummary::CachePolicy(CachePolicySummary {
            lifecycle_state: CacheLifecycleState::Stale,
            reuse_admissibility: CacheReuseAdmissibility::ReuseRefused,
            recompute_reason: Some(RecomputeReason::UpstreamDependencyChanged),
            blocked_reason: None,
            coherence_posture: CacheCoherencePosture::Coherent,
        }),
    )
}

fn cache_blocked_dependency_missing() -> AdmissionEnvelope {
    envelope(
        "evidence/cache/blocked",
        ComputeEvidenceKind::CachePolicy,
        ComputeEvidenceSummary::CachePolicy(CachePolicySummary {
            lifecycle_state: CacheLifecycleState::BlockedDependencyMissing,
            reuse_admissibility: CacheReuseAdmissibility::ReuseRefused,
            recompute_reason: Some(RecomputeReason::UpstreamDependencyChanged),
            blocked_reason: Some(CacheBlockedReason::DependencyMissing),
            coherence_posture: CacheCoherencePosture::DependencyMissing,
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

fn compatibility_blocked() -> AdmissionEnvelope {
    envelope(
        "evidence/compatibility/blocked",
        ComputeEvidenceKind::ContractCompatibilityGate,
        ComputeEvidenceSummary::ContractCompatibilityGate(CompatibilityGateSummary {
            classification: CompatibilityClassification::Breaking,
            gate_blocking: true,
            version_bump_required: true,
            readiness_bump_required: true,
            reasons: vec!["gate_version_changed".to_string()],
        }),
    )
}

fn release_ready() -> AdmissionEnvelope {
    envelope(
        "evidence/release/ready",
        ComputeEvidenceKind::ReleaseReadiness,
        ComputeEvidenceSummary::ReleaseReadiness(ReleaseReadinessSummary {
            readiness: EvidenceReadiness::Ready,
            reasons: vec!["release_ready".to_string()],
        }),
    )
}

fn release_blocked() -> AdmissionEnvelope {
    envelope(
        "evidence/release/blocked",
        ComputeEvidenceKind::ReleaseReadiness,
        ComputeEvidenceSummary::ReleaseReadiness(ReleaseReadinessSummary {
            readiness: EvidenceReadiness::Blocked,
            reasons: vec!["memory_readiness_blocked".to_string()],
        }),
    )
}

fn release_needs_review() -> AdmissionEnvelope {
    envelope(
        "evidence/release/needs-review",
        ComputeEvidenceKind::ReleaseReadiness,
        ComputeEvidenceSummary::ReleaseReadiness(ReleaseReadinessSummary {
            readiness: EvidenceReadiness::NeedsReview,
            reasons: vec!["cuda_promotion_needs_review".to_string()],
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

fn backend_inadmissible() -> AdmissionEnvelope {
    envelope(
        "evidence/backend/inadmissible",
        ComputeEvidenceKind::BackendAdmissibility,
        ComputeEvidenceSummary::BackendAdmissibility {
            admissibility: BackendAdmissibility::Inadmissible,
            reason: "backend_target_inadmissible".to_string(),
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

fn backend_runtime_needs_review() -> AdmissionEnvelope {
    envelope(
        "evidence/backend/runtime-review",
        ComputeEvidenceKind::BackendRuntimePosture,
        ComputeEvidenceSummary::BackendRuntimePosture(BackendRuntimePostureSummary {
            backend_role: BackendRole::Optimized,
            layout_version: "device_layout_shadow_v1".to_string(),
            layout_posture: BackendMemoryLayoutPosture::VersionMismatch,
            precision_posture: PrecisionPosture::ExplicitPolicy,
            canonicalization_posture: BackendCanonicalizationPosture::BackendIndependent,
            parity_oracle: BackendParityOracle::CpuReference,
        }),
    )
}

fn cuda_promotion(
    promotion_posture: CudaPromotionPosture,
    promotion_reason: CudaPromotionReason,
    layout_compatibility: BackendLayoutCompatibility,
    parity_status: CudaParityStatus,
    canonicalization: CudaCanonicalizationPosture,
    observed_delta_units: u64,
) -> AdmissionEnvelope {
    envelope(
        "evidence/cuda/promotion",
        ComputeEvidenceKind::CudaBackendPromotion,
        ComputeEvidenceSummary::CudaBackendPromotion(CudaBackendPromotionSummary {
            backend_admissibility: BackendAdmissibility::Admissible,
            runtime_track: BackendRuntimeTrack::CudaOptimized,
            layout_version: BackendLayoutVersion::CudaDeviceTensorV0,
            transfer_semantics: HostDeviceTransferSemantics::HostDeviceRoundTrip,
            precision_mode: BackendPrecisionMode::Float32Deterministic,
            packing_policy: BackendPackingPolicy::DeviceLocalContiguous,
            canonicalization_boundary: "backend_independent_compute_artifact_v1".to_string(),
            layout_compatibility,
            parity_budget: CudaParityBudget::BoundedUnits { max_delta_units: 2 },
            observed_delta_units,
            canonicalization,
            workload_eligibility: CudaWorkloadEligibility::Eligible,
            eligibility_reason: CudaWorkloadEligibilityReason::EngineWorkloadEligible,
            parity_status,
            promotion_posture,
            promotion_reason,
        }),
    )
}

fn cuda_promotion_promote() -> AdmissionEnvelope {
    cuda_promotion(
        CudaPromotionPosture::Promote,
        CudaPromotionReason::PromotionEligible,
        BackendLayoutCompatibility::Canonicalizable,
        CudaParityStatus::ParityClean,
        CudaCanonicalizationPosture::Canonicalized,
        0,
    )
}

fn cuda_promotion_layout_hold() -> AdmissionEnvelope {
    cuda_promotion(
        CudaPromotionPosture::Hold,
        CudaPromotionReason::LayoutReviewRequired,
        BackendLayoutCompatibility::ReviewRequired,
        CudaParityStatus::ParityClean,
        CudaCanonicalizationPosture::Canonicalized,
        0,
    )
}

fn cuda_promotion_parity_fallback() -> AdmissionEnvelope {
    cuda_promotion(
        CudaPromotionPosture::Fallback,
        CudaPromotionReason::ParityOverBudget,
        BackendLayoutCompatibility::Canonicalizable,
        CudaParityStatus::ParityOverBudget,
        CudaCanonicalizationPosture::Canonicalized,
        3,
    )
}

fn cuda_promotion_noncanonical_fallback() -> AdmissionEnvelope {
    cuda_promotion(
        CudaPromotionPosture::Fallback,
        CudaPromotionReason::NonCanonicalizable,
        BackendLayoutCompatibility::Canonicalizable,
        CudaParityStatus::ParityClean,
        CudaCanonicalizationPosture::NonCanonicalizable,
        0,
    )
}

fn cuda_promotion_backend_inadmissible_fallback() -> AdmissionEnvelope {
    let mut evidence = cuda_promotion(
        CudaPromotionPosture::Fallback,
        CudaPromotionReason::BackendInadmissible,
        BackendLayoutCompatibility::Canonicalizable,
        CudaParityStatus::ParityClean,
        CudaCanonicalizationPosture::Canonicalized,
        0,
    );
    if let ComputeEvidenceSummary::CudaBackendPromotion(summary) = &mut evidence.summary {
        summary.backend_admissibility = BackendAdmissibility::Inadmissible;
    }
    evidence
}

fn work_item(
    key: &str,
    intent: RoutingIntentKind,
    evidence: Vec<AdmissionEnvelope>,
) -> QueueableWorkItem {
    QueueableWorkItem::new(
        key,
        "workflow/orchestration",
        intent,
        QueuePriority::Normal,
        evidence,
    )
    .expect("work item should be valid")
}

#[test]
fn normal_execution_promotes_only_from_typed_ready_evidence() {
    let item = work_item(
        "work/normal-ready",
        RoutingIntentKind::NormalExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::PromoteForExecution
    );
    assert_eq!(audit.state, OrchestrationState::CommandIssued);
    assert_eq!(
        audit.explanation_bundle.admission_records[0].outcome,
        AdmissionOutcome::Admitted
    );
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"release_ready".to_string()));
    assert!(!audit.routing_explanation.control_reason_ids.is_empty());
}

#[test]
fn stale_cache_posture_triggers_recompute_without_recomputing_cache_truth() {
    let item = work_item(
        "work/recompute",
        RoutingIntentKind::NormalExecution,
        vec![
            cache_stale_recompute(),
            compatibility_clean(),
            release_ready(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::TriggerRecompute
    );
    assert_eq!(audit.state, OrchestrationState::RecomputeTriggered);
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CacheRequiresRecompute));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"upstream_dependency_changed".to_string()));
}

#[test]
fn release_blocking_and_compatibility_blocking_prevent_execution() {
    let item = work_item(
        "work/blocked",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            backend_runtime_ready(),
            compatibility_blocked(),
            release_blocked(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(audit.command.command_kind, ExecutionCommandKind::Escalate);
    assert_eq!(audit.state, OrchestrationState::Escalated);
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::ReleaseReadinessBlocked));
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CompatibilityGateBlocking));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"memory_readiness_blocked".to_string()));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"gate_version_changed".to_string()));
}

#[test]
fn accelerated_route_fails_closed_without_pr37_promotion_evidence() {
    let item = work_item(
        "work/accelerated-missing-backend",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            backend_runtime_ready(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::HoldForReview
    );
    assert_eq!(audit.state, OrchestrationState::HeldForReview);
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CudaPromotionEvidenceMissing));
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::AcceleratedRouteRequiresTypedEvidence));
}

#[test]
fn accelerated_release_readiness_needs_review_holds_without_missing_reason() {
    let item = work_item(
        "work/accelerated-release-review",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            backend_admissible(),
            backend_runtime_ready(),
            cache_fresh(),
            compatibility_clean(),
            release_needs_review(),
            cuda_promotion_promote(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::HoldForReview
    );
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::ReleaseReadinessNeedsReview));
    assert!(!audit
        .decision
        .reasons
        .contains(&RoutingReason::ReleaseReadinessMissing));
    assert_ne!(
        audit.command.command_kind,
        ExecutionCommandKind::PromoteForExecution
    );
}

#[test]
fn accelerated_route_with_required_evidence_can_promote() {
    let item = work_item(
        "work/accelerated-ready",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            backend_admissible(),
            backend_runtime_ready(),
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion_promote(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::PromoteForExecution
    );
    assert_eq!(audit.state, OrchestrationState::CommandIssued);
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"promotion_eligible".to_string()));
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CudaPromotionEligible));
}

#[test]
fn accelerated_cuda_layout_review_holds_with_specific_reason() {
    let item = work_item(
        "work/accelerated-layout-review",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion_layout_hold(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::HoldForReview
    );
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CudaLayoutReviewRequired));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"layout_review_required".to_string()));
}

#[test]
fn accelerated_cuda_parity_over_budget_routes_to_fallback() {
    let item = work_item(
        "work/accelerated-parity-over-budget",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion_parity_fallback(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::RouteToFallback
    );
    assert_eq!(audit.state, OrchestrationState::FallbackRouted);
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CudaParityOverBudget));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"parity_over_budget".to_string()));
}

#[test]
fn accelerated_cuda_promotion_posture_beats_standalone_backend_inadmissible() {
    let item = work_item(
        "work/accelerated-backend-inadmissible-fallback",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            backend_inadmissible(),
            cuda_promotion_backend_inadmissible_fallback(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::RouteToFallback
    );
    assert_ne!(
        audit.command.command_kind,
        ExecutionCommandKind::RefuseExecution
    );
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CudaBackendInadmissible));
    assert!(!audit
        .decision
        .reasons
        .contains(&RoutingReason::BackendInadmissible));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"backend_inadmissible".to_string()));
}

#[test]
fn accelerated_cuda_noncanonicalizable_routes_to_fallback() {
    let item = work_item(
        "work/accelerated-noncanonical",
        RoutingIntentKind::AcceleratedExecution,
        vec![
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            cuda_promotion_noncanonical_fallback(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::RouteToFallback
    );
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::CudaNonCanonicalizable));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"non_canonicalizable".to_string()));
}

#[test]
fn normal_execution_is_not_held_solely_for_backend_runtime_review() {
    let item = work_item(
        "work/normal-with-backend-review",
        RoutingIntentKind::NormalExecution,
        vec![
            plugin_compatible(),
            cache_fresh(),
            compatibility_clean(),
            release_ready(),
            backend_runtime_needs_review(),
        ],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::PromoteForExecution
    );
    assert_eq!(audit.state, OrchestrationState::CommandIssued);
    assert!(!audit
        .decision
        .reasons
        .contains(&RoutingReason::BackendRuntimeNeedsParity));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"version_mismatch".to_string()));
}

#[test]
fn blocked_cache_dependency_missing_does_not_trigger_recompute() {
    let item = work_item(
        "work/cache-blocked",
        RoutingIntentKind::NormalExecution,
        vec![cache_blocked_dependency_missing(), release_ready()],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_ne!(
        audit.command.command_kind,
        ExecutionCommandKind::TriggerRecompute
    );
    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::RefuseExecution
    );
    assert_eq!(audit.state, OrchestrationState::Refused);
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::AdmissionRejected));
    assert!(audit
        .routing_explanation
        .compute_reason_ids
        .contains(&"blocked_dependency_missing".to_string()));
}

#[test]
fn malformed_or_unknown_evidence_refuses_execution_without_inference() {
    let mut unknown = release_ready();
    unknown.validation_posture = ComputeValidationPosture::Unknown;

    let admission = admit_evidence(&unknown);
    assert_eq!(admission.outcome, AdmissionOutcome::Rejected);

    let item = work_item(
        "work/unknown-evidence",
        RoutingIntentKind::NormalExecution,
        vec![unknown],
    );

    let audit = route_work_item(&item).expect("routing should succeed");

    assert_eq!(
        audit.command.command_kind,
        ExecutionCommandKind::RefuseExecution
    );
    assert_eq!(audit.state, OrchestrationState::Refused);
    assert!(!audit.routing_explanation.rejected_evidence_keys.is_empty());
    assert!(audit
        .decision
        .reasons
        .contains(&RoutingReason::AdmissionRejected));
}
