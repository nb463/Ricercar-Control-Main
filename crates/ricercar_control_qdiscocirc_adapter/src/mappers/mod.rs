use ricercar_control::{
    execution_command_kind_id, routing_reason_id, AdmissionEnvelope, AdmissionRecord, Disposition,
    ExecutionCommandKind, ExplanationBundle, GovernanceReason, GovernanceRecord,
    OrchestrationAuditRecord, RoutingReason, SystemReleaseGovernanceReason,
    SystemReleaseGovernanceRecord, SystemReleasePosture, TrustClass,
};
use ricercar_control::{system_release_governance_reason_id, system_release_posture_id};

use crate::{
    envelope::{probe_envelope, ProbeEnvelope},
    model::{ProbeEdge, ProbeEdgeKind, ProbeNode, ProbeNodeRole, ProbeTrace, ProbeWalkthroughKind},
    refs::SourceRef,
};

pub fn map_admission(envelope: &AdmissionEnvelope, record: &AdmissionRecord) -> ProbeEnvelope {
    let compute_node = compute_evidence_node(envelope);
    let admission_node = ProbeNode::new(
        "control/admission",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "admission_record",
            record.evidence_key.clone(),
            Some(format!("admission/{}", record.evidence_key)),
            vec![envelope.provenance.artifact_key.clone()],
        ),
        "admission outcome",
        format!(
            "outcome={:?} rejection_reasons={:?}",
            record.outcome, record.rejection_reasons
        ),
        vec!["admission".to_string()],
    );

    probe_envelope(
        format!("probe/control/admission/{}", record.evidence_key),
        "Control admission probe",
        ProbeWalkthroughKind::ShowMeWhy,
        "admission projects compute evidence into Control eligibility without approval",
    )
    .node(compute_node)
    .node(admission_node)
    .edge(ProbeEdge::new(
        "compute/evidence",
        "control/admission",
        ProbeEdgeKind::DrillsDownTo,
        "admission consumes compute evidence",
    ))
    .build()
}

pub fn map_governance(
    envelope: &AdmissionEnvelope,
    admission: &AdmissionRecord,
    governance: &GovernanceRecord,
) -> ProbeEnvelope {
    let mut builder = map_admission(envelope, admission);
    let governance_node = ProbeNode::new(
        "control/governance",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "governance_record",
            governance.evidence_key.clone(),
            Some(format!("governance/{}", governance.evidence_key)),
            vec![admission.evidence_key.clone()],
        ),
        "trust and disposition governance",
        format!(
            "trust={} disposition={} reasons={:?}",
            trust_class_id(governance.trust_class),
            disposition_id(governance.disposition),
            governance.reasons
        ),
        vec!["governance".to_string(), "workflow_truth".to_string()],
    );
    builder.nodes.push(governance_node);
    builder.edges.push(ProbeEdge::new(
        "control/admission",
        "control/governance",
        ProbeEdgeKind::Explains,
        "admitted evidence receives Control-owned trust/disposition",
    ));
    for reason in &governance.reasons {
        if is_blocking_or_review_governance_reason(*reason) {
            builder
                .explanation
                .blocking_reason_ids
                .push(governance_reason_id(*reason).to_string());
        }
    }
    builder
}

pub fn map_explanation_bundle(bundle: &ExplanationBundle) -> ProbeEnvelope {
    let mut builder = probe_envelope(
        format!("probe/control/explanation/{}", bundle.bundle_key),
        "Control explanation probe",
        ProbeWalkthroughKind::ShowMeWhy,
        "explanation bundle projects admission, governance, surfacing, and diagram hints",
    );

    let bundle_node = ProbeNode::new(
        "control/explanation",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "explanation_bundle",
            bundle.bundle_key.clone(),
            Some(bundle.audit_summary.clone()),
            bundle
                .admission_records
                .iter()
                .map(|record| record.evidence_key.clone())
                .collect(),
        ),
        "workflow-auditable explanation bundle",
        format!(
            "trust={} disposition={} fragments={}",
            trust_class_id(bundle.trust_class),
            disposition_id(bundle.disposition),
            bundle.fragments.len()
        ),
        vec!["explanation".to_string(), "surfacing".to_string()],
    );
    builder = builder.node(bundle_node);

    for flow in &bundle.diagram_hint.evidence_flow {
        let evidence_node_id = format!("compute/evidence/{}", flow.evidence_key);
        builder = builder
            .node(ProbeNode::new(
                evidence_node_id.clone(),
                ProbeNodeRole::ComputeTruth,
                SourceRef::compute_evidence(
                    format!("{:?}", flow.evidence_kind),
                    flow.evidence_key.clone(),
                    None,
                    None,
                    flow.rejection_reasons
                        .iter()
                        .map(|reason| format!("admission_rejection:{reason:?}"))
                        .collect(),
                ),
                "compute evidence in control trace",
                format!(
                    "admission={:?} disposition={:?}",
                    flow.admission_outcome, flow.disposition
                ),
                vec!["compute_evidence".to_string()],
            ))
            .edge(ProbeEdge::new(
                evidence_node_id,
                "control/explanation",
                ProbeEdgeKind::DrillsDownTo,
                "explanation preserves evidence flow",
            ));
    }
    for (index, fragment) in bundle.fragments.iter().enumerate() {
        let fragment_node_id = format!("control/explanation-fragment/{index}");
        builder = builder
            .node(ProbeNode::new(
                fragment_node_id.clone(),
                ProbeNodeRole::ControlTruth,
                SourceRef::control_truth(
                    "explanation_fragment",
                    format!("{}/fragment/{index}", bundle.bundle_key),
                    Some(bundle.audit_summary.clone()),
                    vec![fragment.evidence_key.clone()],
                ),
                "explanation fragment",
                format!(
                    "incident={:?} severity={:?} evidence_key={} summary={}",
                    fragment.incident_kind,
                    fragment.severity,
                    fragment.evidence_key,
                    fragment.summary
                ),
                vec!["explanation_fragment".to_string()],
            ))
            .edge(ProbeEdge::new(
                fragment_node_id,
                "control/explanation",
                ProbeEdgeKind::Explains,
                "fragment contributes to explanation bundle",
            ));
    }
    for (index, directive) in bundle.surfacing.iter().enumerate() {
        let surfacing_node_id = format!("control/surfacing/{index}");
        builder = builder
            .node(ProbeNode::new(
                surfacing_node_id.clone(),
                ProbeNodeRole::ControlTruth,
                SourceRef::control_truth(
                    "surfacing_directive",
                    format!("{}/surfacing/{index}", bundle.bundle_key),
                    Some(bundle.audit_summary.clone()),
                    vec![directive.headline.clone()],
                ),
                "surfacing directive",
                format!(
                    "audience={:?} action={:?} headline={} body={}",
                    directive.audience, directive.action, directive.headline, directive.body
                ),
                vec!["surfacing_directive".to_string()],
            ))
            .edge(ProbeEdge::new(
                "control/explanation",
                surfacing_node_id,
                ProbeEdgeKind::Explains,
                "explanation emits this surfacing directive",
            ));
    }
    for (index, step) in bundle.diagram_hint.interpretation_steps.iter().enumerate() {
        let step_node_id = format!("control/interpretation-step/{index}");
        builder = builder
            .node(ProbeNode::new(
                step_node_id.clone(),
                ProbeNodeRole::ControlTruth,
                SourceRef::control_truth(
                    "diagram_interpretation_step",
                    format!("{}/interpretation/{index}", bundle.bundle_key),
                    Some(bundle.audit_summary.clone()),
                    vec![step.evidence_key.clone()],
                ),
                "diagram interpretation step",
                format!(
                    "step={:?} evidence_key={} outcome={:?}",
                    step.step, step.evidence_key, step.outcome
                ),
                vec!["diagram_interpretation_step".to_string()],
            ))
            .edge(ProbeEdge::new(
                step_node_id,
                "control/explanation",
                ProbeEdgeKind::Explains,
                "interpretation step is projected without adding workflow policy",
            ));
    }
    for (index, posture) in bundle.diagram_hint.posture_flow.iter().enumerate() {
        let posture_node_id = format!("control/posture-flow/{index}");
        builder = builder
            .node(ProbeNode::new(
                posture_node_id.clone(),
                ProbeNodeRole::ControlTruth,
                SourceRef::control_truth(
                    "diagram_posture_flow",
                    format!("{}/posture/{index}", bundle.bundle_key),
                    Some(bundle.audit_summary.clone()),
                    vec![posture.evidence_key.clone()],
                ),
                "diagram posture flow",
                format!(
                    "channel={:?} evidence_key={} outcome={:?}",
                    posture.channel, posture.evidence_key, posture.outcome
                ),
                vec!["diagram_posture_flow".to_string()],
            ))
            .edge(ProbeEdge::new(
                posture_node_id,
                "control/explanation",
                ProbeEdgeKind::DrillsDownTo,
                "posture flow preserves Control interpretation of admitted evidence",
            ));
    }

    builder
        .trace(ProbeTrace {
            trace_id: format!("trace/{}", bundle.trace_key),
            title: "Control explanation drill-down".to_string(),
            node_ids: bundle
                .diagram_hint
                .evidence_flow
                .iter()
                .map(|flow| format!("compute/evidence/{}", flow.evidence_key))
                .chain(
                    (0..bundle.fragments.len())
                        .map(|index| format!("control/explanation-fragment/{index}")),
                )
                .chain(
                    (0..bundle.surfacing.len()).map(|index| format!("control/surfacing/{index}")),
                )
                .chain(
                    (0..bundle.diagram_hint.interpretation_steps.len())
                        .map(|index| format!("control/interpretation-step/{index}")),
                )
                .chain(
                    (0..bundle.diagram_hint.posture_flow.len())
                        .map(|index| format!("control/posture-flow/{index}")),
                )
                .chain(std::iter::once("control/explanation".to_string()))
                .collect(),
            summary: "compute evidence -> admission/governance -> explanation/surfacing"
                .to_string(),
        })
        .build()
}

pub fn map_orchestration_audit_record(audit: &OrchestrationAuditRecord) -> ProbeEnvelope {
    let mut builder = probe_envelope(
        format!("probe/control/orchestration/{}", audit.audit_key),
        "Control routing consequence probe",
        walkthrough_for_command(audit.command.command_kind),
        "routing consequence drills down to Control explanation and Compute-owned reason ids",
    );

    let command_node = ProbeNode::new(
        "control/command",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "execution_command",
            audit.command.command_key.clone(),
            Some(audit.audit_key.clone()),
            audit.command.evidence_keys.clone(),
        ),
        "execution command / routing consequence",
        format!(
            "command={} state={:?}",
            execution_command_kind_id(audit.command.command_kind),
            audit.state
        ),
        vec!["workflow_consequence".to_string(), "routing".to_string()],
    );
    let decision_node = ProbeNode::new(
        "control/routing-decision",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "routing_decision",
            audit.decision.decision_key.clone(),
            Some(audit.audit_key.clone()),
            audit.decision.evidence_keys.clone(),
        ),
        "routing decision",
        format!(
            "intent={:?} reasons={}",
            audit.decision.intent,
            audit
                .decision
                .reasons
                .iter()
                .copied()
                .map(routing_reason_id)
                .collect::<Vec<_>>()
                .join(",")
        ),
        vec!["routing_decision".to_string()],
    );
    let explanation_node = ProbeNode::new(
        "control/routing-explanation",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "routing_explanation_payload",
            audit.routing_explanation.explanation_key.clone(),
            Some(audit.routing_explanation.audit_ref.clone()),
            audit.routing_explanation.compute_reason_ids.clone(),
        ),
        "routing explanation payload",
        format!(
            "compute_reasons={} control_reasons={}",
            audit.routing_explanation.compute_reason_ids.join(","),
            audit.routing_explanation.control_reason_ids.join(",")
        ),
        vec!["routing_explanation".to_string()],
    );

    builder = builder
        .node(command_node)
        .node(decision_node)
        .node(explanation_node)
        .edge(ProbeEdge::new(
            "control/command",
            "control/routing-decision",
            ProbeEdgeKind::Explains,
            "command is justified by routing decision",
        ))
        .edge(ProbeEdge::new(
            "control/routing-decision",
            "control/routing-explanation",
            ProbeEdgeKind::DrillsDownTo,
            "decision carries explanation payload",
        ));

    for evidence_key in audit
        .routing_explanation
        .admitted_evidence_keys
        .iter()
        .chain(&audit.routing_explanation.rejected_evidence_keys)
    {
        let node_id = format!("compute/evidence/{evidence_key}");
        builder = builder
            .node(ProbeNode::new(
                node_id.clone(),
                ProbeNodeRole::ComputeTruth,
                SourceRef::compute_evidence(
                    "admitted_or_rejected_compute_evidence",
                    evidence_key.clone(),
                    None,
                    Some(audit.audit_key.clone()),
                    audit.routing_explanation.compute_reason_ids.clone(),
                ),
                "compute evidence consumed by routing",
                format!("evidence_key={evidence_key}"),
                vec!["compute_evidence".to_string()],
            ))
            .edge(ProbeEdge::new(
                node_id,
                "control/routing-decision",
                ProbeEdgeKind::DrillsDownTo,
                "routing consumes this evidence key",
            ));
    }

    for reason in &audit.decision.reasons {
        if is_blocking_routing_reason(*reason) {
            builder = builder.blocking_reason(routing_reason_id(*reason));
        }
    }

    builder
        .trace(ProbeTrace {
            trace_id: format!("trace/{}", audit.audit_key),
            title: "Workflow consequence drill-down".to_string(),
            node_ids: vec![
                "control/command".to_string(),
                "control/routing-decision".to_string(),
                "control/routing-explanation".to_string(),
            ],
            summary:
                "workflow consequence -> routing decision -> explanation -> compute evidence refs"
                    .to_string(),
        })
        .build()
}

pub fn map_system_release_governance_record(
    record: &SystemReleaseGovernanceRecord,
) -> ProbeEnvelope {
    let reason_ids = record
        .reasons
        .iter()
        .copied()
        .map(system_release_governance_reason_id)
        .collect::<Vec<_>>();
    let governance_node = ProbeNode::new(
        "control/system-release-governance",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "system_release_governance_record",
            record.governance_key.clone(),
            Some(record.audit_ref.clone()),
            record.evidence_keys.clone(),
        ),
        "system release governance posture",
        format!(
            "posture={} state={} policy_version={} reasons={}",
            system_release_posture_id(record.posture),
            governance_operational_state_id(record.operational_state),
            record.policy_version.as_deref().unwrap_or("none"),
            reason_ids.join(",")
        ),
        vec![
            "release_governance".to_string(),
            "workflow_consequence".to_string(),
        ],
    );
    let explanation_node = ProbeNode::new(
        "control/release-explanation",
        ProbeNodeRole::ControlTruth,
        SourceRef::control_truth(
            "release_governance_explanation_ref",
            record.explanation_ref.clone(),
            Some(record.audit_ref.clone()),
            record.evidence_keys.clone(),
        ),
        "release governance explanation reference",
        format!(
            "audit_ref={} explanation_ref={}",
            record.audit_ref, record.explanation_ref
        ),
        vec!["release_governance_explanation".to_string()],
    );
    let mut builder = probe_envelope(
        format!("probe/control/release-governance/{}", record.governance_key),
        "Control release governance probe",
        walkthrough_for_release_posture(record.posture),
        "system release governance projects Control consequence over Compute evidence",
    )
    .node(governance_node)
    .node(explanation_node)
    .edge(ProbeEdge::new(
        "control/system-release-governance",
        "control/release-explanation",
        ProbeEdgeKind::Explains,
        "release governance is tied to its explanation/audit reference",
    ));

    for evidence_key in &record.evidence_keys {
        let node_id = format!("compute/evidence/{evidence_key}");
        builder = builder
            .node(ProbeNode::new(
                node_id.clone(),
                ProbeNodeRole::ComputeTruth,
                SourceRef::compute_evidence(
                    "compute_evidence_consumed_by_release_governance",
                    evidence_key.clone(),
                    None,
                    Some(record.audit_ref.clone()),
                    reason_ids
                        .iter()
                        .map(|reason| (*reason).to_string())
                        .collect(),
                ),
                "compute evidence consumed by release governance",
                format!("evidence_key={evidence_key}"),
                vec!["compute_evidence".to_string()],
            ))
            .edge(ProbeEdge::new(
                node_id,
                "control/system-release-governance",
                ProbeEdgeKind::DrillsDownTo,
                "Control release governance consumes this Compute evidence key",
            ));
    }

    for reason in &record.reasons {
        if is_blocking_or_review_release_reason(*reason) {
            builder = builder.blocking_reason(system_release_governance_reason_id(*reason));
        }
    }

    builder
        .trace(ProbeTrace {
            trace_id: format!("trace/{}", record.governance_key),
            title: "System release governance drill-down".to_string(),
            node_ids: record
                .evidence_keys
                .iter()
                .map(|key| format!("compute/evidence/{key}"))
                .chain(std::iter::once(
                    "control/system-release-governance".to_string(),
                ))
                .chain(std::iter::once("control/release-explanation".to_string()))
                .collect(),
            summary: "compute evidence -> Control release governance -> explanation/audit ref"
                .to_string(),
        })
        .build()
}

fn compute_evidence_node(envelope: &AdmissionEnvelope) -> ProbeNode {
    ProbeNode::new(
        "compute/evidence",
        ProbeNodeRole::ComputeTruth,
        SourceRef::compute_evidence(
            format!("{:?}", envelope.evidence_kind),
            envelope.evidence_key.clone(),
            Some(envelope.provenance.content_hash.clone()),
            Some(envelope.provenance.replay_ref.clone()),
            envelope.provenance.lineage.clone(),
        ),
        "compute evidence",
        format!(
            "kind={:?} validation={:?} semantic={:?}",
            envelope.evidence_kind, envelope.validation_posture, envelope.semantic_status
        ),
        vec!["compute_evidence".to_string()],
    )
}

fn trust_class_id(value: TrustClass) -> &'static str {
    match value {
        TrustClass::Ready => "ready",
        TrustClass::ReviewRequired => "review_required",
        TrustClass::Degraded => "degraded",
        TrustClass::FallbackOnly => "fallback_only",
        TrustClass::Refused => "refused",
    }
}

fn disposition_id(value: Disposition) -> &'static str {
    match value {
        Disposition::Promote => "promote",
        Disposition::Fallback => "fallback",
        Disposition::Refuse => "refuse",
        Disposition::Suppress => "suppress",
        Disposition::Degrade => "degrade",
        Disposition::Escalate => "escalate",
        Disposition::HoldForReview => "hold_for_review",
    }
}

fn walkthrough_for_command(command: ExecutionCommandKind) -> ProbeWalkthroughKind {
    match command {
        ExecutionCommandKind::PromoteForExecution => ProbeWalkthroughKind::ShowMeWhy,
        ExecutionCommandKind::HoldForReview
        | ExecutionCommandKind::Escalate
        | ExecutionCommandKind::RefuseExecution
        | ExecutionCommandKind::RouteToFallback
        | ExecutionCommandKind::SuppressOrdinaryRouting => ProbeWalkthroughKind::ShowMeWhy,
        ExecutionCommandKind::TriggerRecompute => ProbeWalkthroughKind::ShowMeWhatChanged,
    }
}

fn is_blocking_routing_reason(reason: RoutingReason) -> bool {
    matches!(
        reason,
        RoutingReason::AdmissionRejected
            | RoutingReason::CompatibilityGateBlocking
            | RoutingReason::ReleaseReadinessBlocked
            | RoutingReason::ReleaseReadinessNeedsReview
            | RoutingReason::ReleaseReadinessMissing
            | RoutingReason::CacheBlocked
            | RoutingReason::CacheRetired
            | RoutingReason::BackendAdmissibilityMissing
            | RoutingReason::BackendInadmissible
            | RoutingReason::BackendRuntimeNeedsParity
            | RoutingReason::AcceleratedRouteRequiresTypedEvidence
    )
}

fn is_blocking_or_review_governance_reason(reason: GovernanceReason) -> bool {
    matches!(
        reason,
        GovernanceReason::AdmissionRejected
            | GovernanceReason::EvidenceDegraded
            | GovernanceReason::ComputeRefused
            | GovernanceReason::PluginIncompatible
            | GovernanceReason::BackendInadmissible
            | GovernanceReason::CacheStale
            | GovernanceReason::CacheRetired
            | GovernanceReason::CompatibilityGateBlocking
            | GovernanceReason::CompatibilityGateNeedsReview
            | GovernanceReason::ReadinessNeedsReview
            | GovernanceReason::ReadinessBlocked
            | GovernanceReason::BackendRuntimeNeedsParity
            | GovernanceReason::CudaPromotionNeedsReview
            | GovernanceReason::CudaPromotionDegraded
            | GovernanceReason::CudaPromotionFallback
            | GovernanceReason::GenericArtifactNeedsReview
            | GovernanceReason::EvidenceNonComparable
    )
}

fn walkthrough_for_release_posture(posture: SystemReleasePosture) -> ProbeWalkthroughKind {
    match posture {
        SystemReleasePosture::Promotable => ProbeWalkthroughKind::ShowMeWhy,
        SystemReleasePosture::HoldForReview
        | SystemReleasePosture::DegradedButGovernable
        | SystemReleasePosture::FallbackOnly
        | SystemReleasePosture::RollbackRequired
        | SystemReleasePosture::Blocked => ProbeWalkthroughKind::ShowMeWhatBlockedPromotion,
    }
}

fn is_blocking_or_review_release_reason(reason: SystemReleaseGovernanceReason) -> bool {
    matches!(
        reason,
        SystemReleaseGovernanceReason::ControlReadinessNeedsReview
            | SystemReleaseGovernanceReason::ControlReadinessBlocked
            | SystemReleaseGovernanceReason::ComputeCompatibilityGateBlocking
            | SystemReleaseGovernanceReason::ComputeReleaseNeedsReview
            | SystemReleaseGovernanceReason::ComputeReleaseBlocked
            | SystemReleaseGovernanceReason::BackendRolloutHold
            | SystemReleaseGovernanceReason::BackendRolloutDegraded
            | SystemReleaseGovernanceReason::BackendRolloutFallbackOnly
            | SystemReleaseGovernanceReason::IncidentOperationalHold
            | SystemReleaseGovernanceReason::IncidentDegradedOperation
            | SystemReleaseGovernanceReason::IncidentEscalationRequired
            | SystemReleaseGovernanceReason::IncidentRollbackRequired
            | SystemReleaseGovernanceReason::IncidentRollbackInEffect
            | SystemReleaseGovernanceReason::IncidentBlocked
    )
}

fn governance_operational_state_id(
    state: ricercar_control::GovernanceOperationalState,
) -> &'static str {
    match state {
        ricercar_control::GovernanceOperationalState::Promoted => "promoted",
        ricercar_control::GovernanceOperationalState::Held => "held",
        ricercar_control::GovernanceOperationalState::Degraded => "degraded",
        ricercar_control::GovernanceOperationalState::FallbackOnly => "fallback_only",
        ricercar_control::GovernanceOperationalState::Escalated => "escalated",
        ricercar_control::GovernanceOperationalState::RollbackRequired => "rollback_required",
        ricercar_control::GovernanceOperationalState::RollbackInEffect => "rollback_in_effect",
        ricercar_control::GovernanceOperationalState::Blocked => "blocked",
    }
}

fn governance_reason_id(reason: GovernanceReason) -> &'static str {
    match reason {
        GovernanceReason::AdmissionRejected => "admission_rejected",
        GovernanceReason::EvidenceReady => "evidence_ready",
        GovernanceReason::EvidenceDegraded => "evidence_degraded",
        GovernanceReason::ComputeRefused => "compute_refused",
        GovernanceReason::BackendAdmissible => "backend_admissible",
        GovernanceReason::PluginCompatible => "plugin_compatible",
        GovernanceReason::PluginIncompatible => "plugin_incompatible",
        GovernanceReason::BackendInadmissible => "backend_inadmissible",
        GovernanceReason::CacheFresh => "cache_fresh",
        GovernanceReason::CacheStale => "cache_stale",
        GovernanceReason::CacheRetired => "cache_retired",
        GovernanceReason::CompatibilityGateClean => "compatibility_gate_clean",
        GovernanceReason::CompatibilityGateBlocking => "compatibility_gate_blocking",
        GovernanceReason::CompatibilityGateNeedsReview => "compatibility_gate_needs_review",
        GovernanceReason::ReadinessNeedsReview => "readiness_needs_review",
        GovernanceReason::ReadinessBlocked => "readiness_blocked",
        GovernanceReason::BackendRuntimeReady => "backend_runtime_ready",
        GovernanceReason::BackendRuntimeNeedsParity => "backend_runtime_needs_parity",
        GovernanceReason::CudaPromotionEligible => "cuda_promotion_eligible",
        GovernanceReason::CudaPromotionNeedsReview => "cuda_promotion_needs_review",
        GovernanceReason::CudaPromotionDegraded => "cuda_promotion_degraded",
        GovernanceReason::CudaPromotionFallback => "cuda_promotion_fallback",
        GovernanceReason::GenericArtifactNeedsReview => "generic_artifact_needs_review",
        GovernanceReason::EvidenceNonComparable => "evidence_non_comparable",
    }
}
