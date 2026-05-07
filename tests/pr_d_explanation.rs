use ricercar_control::{
    admit_evidence, assemble_explanation_bundle, AdmissionEnvelope, AdmissionOutcome,
    AdmissionRejectionReason, BackendAdmissibility, BackendCanonicalizationPosture,
    BackendMemoryLayoutPosture, BackendRole, BackendRuntimePostureSummary, CacheCoherencePosture,
    CacheLifecycleState, CachePolicySummary, CacheReuseAdmissibility, CompatibilityClassification,
    CompatibilityGateSummary, ComputeEvidenceKind, ComputeEvidenceSummary, ComputeSemanticStatus,
    ComputeValidationPosture, ControlTrace, Disposition, EvidenceProvenance, EvidenceReadiness,
    IncidentKind, PluginCompatibility, PluginCompatibilityReason, PluginCompatibilitySummary,
    PrecisionPosture, ReleaseReadinessSummary, SurfacingAction, SurfacingAudience, TrustClass,
};

fn hash(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn provenance(artifact_key: &str) -> EvidenceProvenance {
    EvidenceProvenance::new(
        "ricercar-compute",
        "workflow/triage",
        artifact_key,
        hash('a'),
        "replay/compute/run-1",
        vec![
            "backend_capability".to_string(),
            "plugin_compatibility".to_string(),
            "cache_recompute_posture".to_string(),
        ],
    )
}

fn envelope(
    evidence_key: &str,
    evidence_kind: ComputeEvidenceKind,
    semantic_status: ComputeSemanticStatus,
    summary: ComputeEvidenceSummary,
) -> AdmissionEnvelope {
    AdmissionEnvelope::new(
        evidence_key,
        evidence_kind,
        provenance(evidence_key),
        ComputeValidationPosture::Validated,
        semantic_status,
        summary,
    )
}

fn plugin_summary(compatibility: PluginCompatibility) -> ComputeEvidenceSummary {
    ComputeEvidenceSummary::PluginCompatibility(PluginCompatibilitySummary {
        plugin_key: "plugin/backend/canonical".to_string(),
        boundary_kind: "backend_capability".to_string(),
        operation_kind: "backend_capability".to_string(),
        compatibility,
        reason: if compatibility == PluginCompatibility::Compatible {
            PluginCompatibilityReason::DeclaredCompatible
        } else {
            PluginCompatibilityReason::BackendCapabilityInadmissible
        },
        backend_admissibility: Some(if compatibility == PluginCompatibility::Compatible {
            BackendAdmissibility::Admissible
        } else {
            BackendAdmissibility::Inadmissible
        }),
    })
}

fn fresh_cache_summary() -> ComputeEvidenceSummary {
    ComputeEvidenceSummary::CachePolicy(CachePolicySummary {
        lifecycle_state: CacheLifecycleState::Fresh,
        reuse_admissibility: CacheReuseAdmissibility::ReuseAdmissible,
        recompute_reason: None,
        blocked_reason: None,
        coherence_posture: CacheCoherencePosture::Coherent,
    })
}

#[test]
fn admission_rejects_malformed_or_incomplete_evidence() {
    let mut bad_provenance = provenance("");
    bad_provenance.content_hash = "not-a-hash".to_string();
    bad_provenance.replay_ref.clear();
    bad_provenance.lineage.clear();

    let envelope = AdmissionEnvelope::new(
        "",
        ComputeEvidenceKind::PluginCompatibility,
        bad_provenance,
        ComputeValidationPosture::Malformed,
        ComputeSemanticStatus::Lawful,
        plugin_summary(PluginCompatibility::Compatible),
    );

    let record = admit_evidence(&envelope);
    assert_eq!(record.outcome, AdmissionOutcome::Rejected);
    assert!(record
        .rejection_reasons
        .contains(&AdmissionRejectionReason::MissingEvidenceKey));
    assert!(record
        .rejection_reasons
        .contains(&AdmissionRejectionReason::MissingArtifactIdentity));
    assert!(record
        .rejection_reasons
        .contains(&AdmissionRejectionReason::MissingReplayReference));
    assert!(record
        .rejection_reasons
        .contains(&AdmissionRejectionReason::MissingLineage));
    assert!(record
        .rejection_reasons
        .contains(&AdmissionRejectionReason::MalformedContentHash));
    assert!(record
        .rejection_reasons
        .contains(&AdmissionRejectionReason::ComputeValidationFailed));
}

#[test]
fn plugin_incompatibility_is_admitted_then_refused_by_control() {
    let envelope = envelope(
        "evidence/plugin/incompatible",
        ComputeEvidenceKind::PluginCompatibility,
        ComputeSemanticStatus::Lawful,
        plugin_summary(PluginCompatibility::Incompatible),
    );
    let trace = ControlTrace::new(
        "trace/plugin-incompatible",
        "workflow/triage",
        vec![envelope],
    )
    .expect("trace should be valid");

    let bundle = assemble_explanation_bundle(&trace).expect("bundle should assemble");

    assert_eq!(
        bundle.admission_records[0].outcome,
        AdmissionOutcome::Admitted
    );
    assert_eq!(bundle.trust_class, TrustClass::Refused);
    assert_eq!(bundle.disposition, Disposition::Refuse);
    assert!(bundle
        .fragments
        .iter()
        .any(|fragment| fragment.incident_kind == IncidentKind::PluginIncompatible));
    assert!(bundle.surfacing.iter().any(|directive| {
        directive.audience == SurfacingAudience::Operator
            && directive.action == SurfacingAction::Refuse
    }));
}

#[test]
fn stale_cache_policy_degrades_workflow_surfacing() {
    let envelope = envelope(
        "evidence/cache/stale",
        ComputeEvidenceKind::CachePolicy,
        ComputeSemanticStatus::Degraded,
        ComputeEvidenceSummary::CachePolicy(CachePolicySummary {
            lifecycle_state: CacheLifecycleState::Stale,
            reuse_admissibility: CacheReuseAdmissibility::ReuseRefused,
            recompute_reason: Some(ricercar_control::RecomputeReason::UpstreamDependencyChanged),
            blocked_reason: None,
            coherence_posture: CacheCoherencePosture::Coherent,
        }),
    );
    let trace = ControlTrace::new("trace/cache-stale", "workflow/triage", vec![envelope])
        .expect("trace should be valid");

    let bundle = assemble_explanation_bundle(&trace).expect("bundle should assemble");

    assert_eq!(bundle.trust_class, TrustClass::Degraded);
    assert_eq!(bundle.disposition, Disposition::Degrade);
    assert!(bundle
        .fragments
        .iter()
        .any(|fragment| fragment.incident_kind == IncidentKind::CacheNotReusable));
}

#[test]
fn compatibility_gate_breaking_escalates_with_explanation() {
    let envelope = envelope(
        "evidence/compatibility/breaking",
        ComputeEvidenceKind::ContractCompatibilityGate,
        ComputeSemanticStatus::Lawful,
        ComputeEvidenceSummary::ContractCompatibilityGate(CompatibilityGateSummary {
            classification: CompatibilityClassification::Breaking,
            gate_blocking: true,
            version_bump_required: true,
            readiness_bump_required: true,
            reasons: vec!["gate_version_changed".to_string()],
        }),
    );
    let trace = ControlTrace::new("trace/compat-breaking", "workflow/triage", vec![envelope])
        .expect("trace should be valid");

    let bundle = assemble_explanation_bundle(&trace).expect("bundle should assemble");

    assert_eq!(bundle.trust_class, TrustClass::ReviewRequired);
    assert_eq!(bundle.disposition, Disposition::Escalate);
    assert!(bundle
        .fragments
        .iter()
        .any(|fragment| fragment.incident_kind == IncidentKind::BoundaryDrift));
    assert!(bundle
        .diagram_hint
        .boxes
        .contains(&"compatibility_verdict".to_string()));
}

#[test]
fn backend_runtime_posture_holds_cuda_transition_for_parity_review() {
    let envelope = envelope(
        "evidence/backend/cuda-shadow",
        ComputeEvidenceKind::BackendRuntimePosture,
        ComputeSemanticStatus::Lawful,
        ComputeEvidenceSummary::BackendRuntimePosture(BackendRuntimePostureSummary {
            backend_role: BackendRole::Optimized,
            layout_version: "device_layout_shadow_v1".to_string(),
            layout_posture: BackendMemoryLayoutPosture::VersionMismatch,
            precision_posture: PrecisionPosture::ExplicitPolicy,
            canonicalization_posture: BackendCanonicalizationPosture::BackendIndependent,
            parity_oracle: "cpu_reference_required".to_string(),
        }),
    );
    let trace = ControlTrace::new("trace/backend-runtime", "workflow/triage", vec![envelope])
        .expect("trace should be valid");

    let bundle = assemble_explanation_bundle(&trace).expect("bundle should assemble");

    assert_eq!(bundle.trust_class, TrustClass::ReviewRequired);
    assert_eq!(bundle.disposition, Disposition::HoldForReview);
    assert!(bundle
        .fragments
        .iter()
        .any(|fragment| fragment.incident_kind == IncidentKind::BackendRuntimeNeedsParity));
}

#[test]
fn full_ready_evidence_chain_promotes_with_auditable_bundle() {
    let envelopes = vec![
        envelope(
            "evidence/backend/admissible",
            ComputeEvidenceKind::BackendAdmissibility,
            ComputeSemanticStatus::Lawful,
            ComputeEvidenceSummary::BackendAdmissibility {
                admissibility: BackendAdmissibility::Admissible,
                reason: "declared_compatible".to_string(),
            },
        ),
        envelope(
            "evidence/plugin/compatible",
            ComputeEvidenceKind::PluginCompatibility,
            ComputeSemanticStatus::Lawful,
            plugin_summary(PluginCompatibility::Compatible),
        ),
        envelope(
            "evidence/cache/fresh",
            ComputeEvidenceKind::CachePolicy,
            ComputeSemanticStatus::Lawful,
            fresh_cache_summary(),
        ),
        envelope(
            "evidence/compatibility/internal-only",
            ComputeEvidenceKind::ContractCompatibilityGate,
            ComputeSemanticStatus::Lawful,
            ComputeEvidenceSummary::ContractCompatibilityGate(CompatibilityGateSummary {
                classification: CompatibilityClassification::InternalOnly,
                gate_blocking: false,
                version_bump_required: false,
                readiness_bump_required: false,
                reasons: vec!["no_boundary_drift".to_string()],
            }),
        ),
        envelope(
            "evidence/release/ready",
            ComputeEvidenceKind::ReleaseReadiness,
            ComputeSemanticStatus::Lawful,
            ComputeEvidenceSummary::ReleaseReadiness(ReleaseReadinessSummary {
                readiness: EvidenceReadiness::Ready,
                boundary_readiness_signal: "ready".to_string(),
            }),
        ),
        envelope(
            "evidence/backend/reference",
            ComputeEvidenceKind::BackendRuntimePosture,
            ComputeSemanticStatus::Lawful,
            ComputeEvidenceSummary::BackendRuntimePosture(BackendRuntimePostureSummary {
                backend_role: BackendRole::Reference,
                layout_version: "host_canonical_v0".to_string(),
                layout_posture: BackendMemoryLayoutPosture::HostCanonical,
                precision_posture: PrecisionPosture::DeterministicReference,
                canonicalization_posture: BackendCanonicalizationPosture::BackendIndependent,
                parity_oracle: "cpu_reference".to_string(),
            }),
        ),
    ];
    let trace = ControlTrace::new("trace/full-chain-ready", "workflow/triage", envelopes).unwrap();

    let bundle = assemble_explanation_bundle(&trace).expect("bundle should assemble");

    assert_eq!(bundle.admission_records.len(), 6);
    assert!(bundle
        .admission_records
        .iter()
        .all(|record| record.outcome == AdmissionOutcome::Admitted));
    assert_eq!(bundle.trust_class, TrustClass::Ready);
    assert_eq!(bundle.disposition, Disposition::Promote);
    assert!(bundle.audit_summary.contains("evidence_count=6"));
    assert!(bundle
        .diagram_hint
        .contractions
        .contains(&"evidence_chain_to_explanation_bundle".to_string()));
}
