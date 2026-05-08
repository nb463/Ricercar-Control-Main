use ricercar_control::{
    control_release_readiness_report, evaluate_governance_trace_scenario,
    evaluate_governance_transition, evaluate_system_release_governance, route_work_item,
    system_release_governance_reason_id, AdmissionEnvelope, BackendAdmissibility,
    BackendLayoutCompatibility, BackendLayoutVersion, BackendPackingPolicy, BackendPrecisionMode,
    BackendRuntimeTrack, CacheCoherencePosture, CacheLifecycleState, CachePolicySummary,
    CacheReuseAdmissibility, CompatibilityClassification, CompatibilityGateSummary,
    ComputeEvidenceKind, ComputeEvidenceSummary, ComputeSemanticStatus, ComputeValidationPosture,
    ControlReleaseReadinessInput, ControlReleaseReadinessReason, ControlReleaseReadinessStatus,
    CudaBackendPromotionSummary, CudaCanonicalizationPosture, CudaParityBudget, CudaParityStatus,
    CudaPromotionPosture, CudaPromotionReason, CudaWorkloadEligibility,
    CudaWorkloadEligibilityReason, EvidenceProvenance, EvidenceReadiness, GovernanceIncident,
    GovernanceIncidentKind, GovernanceIncidentResponse, GovernanceOperationalState,
    GovernancePolicySet, GovernanceTraceScenario, GovernanceTransitionGuardOutcome,
    GovernanceTransitionGuardReason, GovernanceTransitionRequest, HostDeviceTransferSemantics,
    OrchestrationAuditRecord, PolicyCompatibilityPosture, QueuePriority, QueueableWorkItem,
    ReleaseReadinessSummary, RoutingIntentKind, SystemReleaseGovernanceInput,
    SystemReleaseGovernanceReason, SystemReleasePosture,
};

fn hash(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn provenance(artifact_key: &str) -> EvidenceProvenance {
    EvidenceProvenance::new(
        "ricercar-compute",
        "workflow/release-governance",
        artifact_key,
        hash('f'),
        "replay/compute/release-governance-1",
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

fn release_readiness(readiness: EvidenceReadiness, reason: &str) -> AdmissionEnvelope {
    envelope(
        &format!("evidence/release/{readiness:?}"),
        ComputeEvidenceKind::ReleaseReadiness,
        ComputeEvidenceSummary::ReleaseReadiness(ReleaseReadinessSummary {
            readiness,
            reasons: vec![reason.to_string()],
        }),
    )
}

fn cuda_promotion(
    posture: CudaPromotionPosture,
    reason: CudaPromotionReason,
    parity: CudaParityStatus,
) -> AdmissionEnvelope {
    envelope(
        &format!("evidence/cuda/{posture:?}"),
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
            observed_delta_units: if parity == CudaParityStatus::ParityClean {
                0
            } else {
                1
            },
            canonicalization: CudaCanonicalizationPosture::Canonicalized,
            workload_eligibility: CudaWorkloadEligibility::Eligible,
            eligibility_reason: CudaWorkloadEligibilityReason::EngineWorkloadEligible,
            parity_status: parity,
            promotion_posture: posture,
            promotion_reason: reason,
        }),
    )
}

fn policy(posture: PolicyCompatibilityPosture) -> GovernancePolicySet {
    GovernancePolicySet::new(
        "policy/control/release-governance",
        "control-policy-v1",
        posture,
        if posture == PolicyCompatibilityPosture::Compatible {
            Vec::new()
        } else {
            vec!["policy_version_moved".to_string()]
        },
    )
    .expect("policy should validate")
}

fn ready_control_readiness() -> ricercar_control::ControlReleaseReadinessReport {
    control_release_readiness_report(ControlReleaseReadinessInput {
        readiness_key: "control-readiness/ready".to_string(),
        policy_set: Some(policy(PolicyCompatibilityPosture::Compatible)),
        audit_requirements_satisfied: true,
        governance_trace_corpus_green: true,
        workflow_transition_guards_present: true,
        upstream_compute_evidence_consumable: true,
        rollback_doctrine_present: true,
        replay_note: "control release readiness is fixture-derived".to_string(),
    })
    .expect("control readiness should evaluate")
}

fn audit_for(key: &str, evidence: Vec<AdmissionEnvelope>) -> OrchestrationAuditRecord {
    route_work_item(
        &QueueableWorkItem::new(
            key,
            "workflow/release-governance",
            RoutingIntentKind::AcceleratedExecution,
            QueuePriority::Normal,
            evidence,
        )
        .expect("work item should validate"),
    )
    .expect("routing should assemble")
}

fn governance_input(
    key: &str,
    evidence: Vec<AdmissionEnvelope>,
    control_readiness: ricercar_control::ControlReleaseReadinessReport,
    incident: Option<GovernanceIncident>,
) -> SystemReleaseGovernanceInput {
    let audit = audit_for(key, evidence.clone());
    SystemReleaseGovernanceInput {
        governance_key: format!("governance/{key}"),
        control_readiness,
        compute_evidence: evidence,
        orchestration_audit: audit,
        incident,
        replay_note: "system governance posture assembled from Control-owned inputs".to_string(),
    }
}

#[test]
fn control_release_readiness_is_ready_when_operational_inputs_are_satisfied() {
    let report = ready_control_readiness();

    assert_eq!(report.status, ControlReleaseReadinessStatus::Ready);
    assert!(report
        .reasons
        .contains(&ControlReleaseReadinessReason::PolicyCompatible));
    assert!(report
        .reasons
        .contains(&ControlReleaseReadinessReason::AuditRequirementsSatisfied));
    assert_eq!(report.policy_version, Some("control-policy-v1".to_string()));
}

#[test]
fn policy_version_compatibility_review_is_explicit_and_auditable() {
    let review = control_release_readiness_report(ControlReleaseReadinessInput {
        readiness_key: "control-readiness/policy-review".to_string(),
        policy_set: Some(policy(PolicyCompatibilityPosture::ReviewRequired)),
        audit_requirements_satisfied: true,
        governance_trace_corpus_green: true,
        workflow_transition_guards_present: true,
        upstream_compute_evidence_consumable: true,
        rollback_doctrine_present: true,
        replay_note: "policy movement requires review".to_string(),
    })
    .expect("review report should evaluate");
    let breaking = control_release_readiness_report(ControlReleaseReadinessInput {
        readiness_key: "control-readiness/policy-breaking".to_string(),
        policy_set: Some(policy(PolicyCompatibilityPosture::Breaking)),
        audit_requirements_satisfied: true,
        governance_trace_corpus_green: true,
        workflow_transition_guards_present: true,
        upstream_compute_evidence_consumable: true,
        rollback_doctrine_present: true,
        replay_note: "policy movement is breaking".to_string(),
    })
    .expect("breaking report should evaluate");

    assert_eq!(review.status, ControlReleaseReadinessStatus::NeedsReview);
    assert!(review
        .reasons
        .contains(&ControlReleaseReadinessReason::PolicyReviewRequired));
    assert_eq!(breaking.status, ControlReleaseReadinessStatus::Blocked);
    assert!(breaking
        .reasons
        .contains(&ControlReleaseReadinessReason::PolicyBreaking));
}

#[test]
fn governance_trace_corpus_covers_promotion_hold_rollback_incompatibility_and_degrade() {
    let ready = ready_control_readiness();
    let promotion = vec![
        cache_fresh(),
        compatibility_clean(),
        release_readiness(EvidenceReadiness::Ready, "release_ready"),
        cuda_promotion(
            CudaPromotionPosture::Promote,
            CudaPromotionReason::PromotionEligible,
            CudaParityStatus::ParityClean,
        ),
    ];
    let hold = vec![
        cache_fresh(),
        compatibility_clean(),
        release_readiness(
            EvidenceReadiness::NeedsReview,
            "cuda_promotion_needs_review",
        ),
        cuda_promotion(
            CudaPromotionPosture::Hold,
            CudaPromotionReason::LayoutReviewRequired,
            CudaParityStatus::ParityClean,
        ),
    ];
    let incompatibility = vec![
        cache_fresh(),
        compatibility_blocked(),
        release_readiness(EvidenceReadiness::Ready, "release_ready"),
        cuda_promotion(
            CudaPromotionPosture::Promote,
            CudaPromotionReason::PromotionEligible,
            CudaParityStatus::ParityClean,
        ),
    ];
    let degraded = vec![
        cache_fresh(),
        compatibility_clean(),
        release_readiness(EvidenceReadiness::Ready, "release_ready"),
        cuda_promotion(
            CudaPromotionPosture::Degrade,
            CudaPromotionReason::ParityWithinBudget,
            CudaParityStatus::ParityWithinBudget,
        ),
    ];
    let rollback = GovernanceIncident::new(
        "incident/rollback",
        GovernanceIncidentKind::BackendLayoutIncident,
        GovernanceIncidentResponse::RollbackRequired,
        "accelerated rollout incident requires rollback",
    )
    .expect("incident should validate");
    let escalation = GovernanceIncident::new(
        "incident/escalation",
        GovernanceIncidentKind::PolicyIncident,
        GovernanceIncidentResponse::EscalationRequired,
        "policy operator review requires escalation",
    )
    .expect("incident should validate");

    let scenarios = vec![
        GovernanceTraceScenario {
            scenario_key: "scenario/promotion".to_string(),
            input: governance_input("scenario-promotion", promotion, ready.clone(), None),
            expected_posture: SystemReleasePosture::Promotable,
            expected_state: GovernanceOperationalState::Promoted,
            required_reason: SystemReleaseGovernanceReason::BackendRolloutReady,
        },
        GovernanceTraceScenario {
            scenario_key: "scenario/hold".to_string(),
            input: governance_input("scenario-hold", hold, ready.clone(), None),
            expected_posture: SystemReleasePosture::HoldForReview,
            expected_state: GovernanceOperationalState::Held,
            required_reason: SystemReleaseGovernanceReason::ComputeReleaseNeedsReview,
        },
        GovernanceTraceScenario {
            scenario_key: "scenario/incompatibility".to_string(),
            input: governance_input(
                "scenario-incompatibility",
                incompatibility,
                ready.clone(),
                None,
            ),
            expected_posture: SystemReleasePosture::Blocked,
            expected_state: GovernanceOperationalState::Blocked,
            required_reason: SystemReleaseGovernanceReason::ComputeCompatibilityGateBlocking,
        },
        GovernanceTraceScenario {
            scenario_key: "scenario/degraded".to_string(),
            input: governance_input("scenario-degraded", degraded, ready.clone(), None),
            expected_posture: SystemReleasePosture::DegradedButGovernable,
            expected_state: GovernanceOperationalState::Degraded,
            required_reason: SystemReleaseGovernanceReason::BackendRolloutDegraded,
        },
        GovernanceTraceScenario {
            scenario_key: "scenario/rollback".to_string(),
            input: governance_input(
                "scenario-rollback",
                vec![
                    cache_fresh(),
                    compatibility_clean(),
                    release_readiness(EvidenceReadiness::Ready, "release_ready"),
                    cuda_promotion(
                        CudaPromotionPosture::Promote,
                        CudaPromotionReason::PromotionEligible,
                        CudaParityStatus::ParityClean,
                    ),
                ],
                ready,
                Some(rollback),
            ),
            expected_posture: SystemReleasePosture::RollbackRequired,
            expected_state: GovernanceOperationalState::RollbackRequired,
            required_reason: SystemReleaseGovernanceReason::IncidentRollbackRequired,
        },
        GovernanceTraceScenario {
            scenario_key: "scenario/escalation".to_string(),
            input: governance_input(
                "scenario-escalation",
                vec![
                    cache_fresh(),
                    compatibility_clean(),
                    release_readiness(EvidenceReadiness::Ready, "release_ready"),
                    cuda_promotion(
                        CudaPromotionPosture::Promote,
                        CudaPromotionReason::PromotionEligible,
                        CudaParityStatus::ParityClean,
                    ),
                ],
                ready_control_readiness(),
                Some(escalation),
            ),
            expected_posture: SystemReleasePosture::HoldForReview,
            expected_state: GovernanceOperationalState::Escalated,
            required_reason: SystemReleaseGovernanceReason::IncidentEscalationRequired,
        },
    ];

    for scenario in scenarios {
        let record = evaluate_governance_trace_scenario(scenario)
            .expect("scenario should match its expected posture");
        assert!(!record.audit_ref.is_empty());
        assert!(!record.explanation_ref.is_empty());
        assert!(record
            .reasons
            .contains(&SystemReleaseGovernanceReason::AuditComplete));
    }
}

#[test]
fn transition_guards_preserve_audit_and_prevent_uncontrolled_repromotion() {
    let missing_audit = evaluate_governance_transition(GovernanceTransitionRequest {
        from: SystemReleasePosture::HoldForReview,
        to: SystemReleasePosture::Promotable,
        audit_complete: false,
        explanation_present: true,
        evidence_keys: vec!["evidence/release/ready".to_string()],
        explicit_review_recorded: true,
    });
    let uncontrolled_promote = evaluate_governance_transition(GovernanceTransitionRequest {
        from: SystemReleasePosture::FallbackOnly,
        to: SystemReleasePosture::Promotable,
        audit_complete: true,
        explanation_present: true,
        evidence_keys: vec!["evidence/cuda/fallback".to_string()],
        explicit_review_recorded: false,
    });
    let reviewed_promote = evaluate_governance_transition(GovernanceTransitionRequest {
        from: SystemReleasePosture::FallbackOnly,
        to: SystemReleasePosture::Promotable,
        audit_complete: true,
        explanation_present: true,
        evidence_keys: vec!["evidence/cuda/fallback".to_string()],
        explicit_review_recorded: true,
    });

    assert_eq!(
        missing_audit.outcome,
        GovernanceTransitionGuardOutcome::Blocked
    );
    assert!(missing_audit
        .reasons
        .contains(&GovernanceTransitionGuardReason::AuditIncomplete));
    assert_eq!(
        uncontrolled_promote.outcome,
        GovernanceTransitionGuardOutcome::ReviewRequired
    );
    assert!(uncontrolled_promote
        .reasons
        .contains(&GovernanceTransitionGuardReason::PromotionRequiresExplicitReview));
    assert_eq!(
        reviewed_promote.outcome,
        GovernanceTransitionGuardOutcome::Allowed
    );
}

#[test]
fn system_governance_preserves_stable_reason_ids_for_downstream_audit() {
    let evidence = vec![
        cache_fresh(),
        compatibility_clean(),
        release_readiness(EvidenceReadiness::Ready, "release_ready"),
        cuda_promotion(
            CudaPromotionPosture::Degrade,
            CudaPromotionReason::ParityWithinBudget,
            CudaParityStatus::ParityWithinBudget,
        ),
    ];
    let record = evaluate_system_release_governance(governance_input(
        "stable-ids",
        evidence,
        ready_control_readiness(),
        None,
    ))
    .expect("governance should evaluate");

    assert_eq!(record.posture, SystemReleasePosture::DegradedButGovernable);
    assert!(record
        .reasons
        .iter()
        .copied()
        .map(system_release_governance_reason_id)
        .any(|reason| reason == "backend_rollout_degraded"));
    assert_eq!(record.policy_version, Some("control-policy-v1".to_string()));
    assert!(record
        .evidence_keys
        .contains(&"evidence/cuda/Degrade".to_string()));
    assert!(record.explanation_ref.contains("routing_explanation/"));
}
