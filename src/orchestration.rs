use crate::admission::{AdmissionEnvelope, AdmissionOutcome};
use crate::evidence::{
    BackendAdmissibility, BackendCanonicalizationPosture, BackendLayoutCompatibility,
    BackendLayoutVersion, BackendMemoryLayoutPosture, BackendPackingPolicy, BackendParityOracle,
    BackendPrecisionMode, BackendRuntimeTrack, CacheBlockedReason, CacheCoherencePosture,
    CacheLifecycleState, CachePolicySummary, CacheReuseAdmissibility, CompatibilityClassification,
    ComputeEvidenceSummary, CudaBackendPromotionSummary, CudaCanonicalizationPosture,
    CudaParityBudget, CudaParityStatus, CudaPromotionPosture, CudaPromotionReason,
    CudaWorkloadEligibility, CudaWorkloadEligibilityReason, EvidenceReadiness,
    HostDeviceTransferSemantics, PluginCompatibility, PluginCompatibilityReason, PrecisionPosture,
    RecomputeReason,
};
use crate::explanation::{assemble_explanation_bundle, ControlTrace, ExplanationBundle};
use crate::governance::{Disposition, GovernanceReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingIntentKind {
    NormalExecution,
    AcceleratedExecution,
    FallbackExecution,
    Recompute,
    Suppress,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueuePriority {
    Normal,
    Elevated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueableWorkItem {
    pub work_item_key: String,
    pub workflow_context: String,
    pub intent: RoutingIntentKind,
    pub priority: QueuePriority,
    pub evidence: Vec<AdmissionEnvelope>,
}

impl QueueableWorkItem {
    pub fn new(
        work_item_key: impl Into<String>,
        workflow_context: impl Into<String>,
        intent: RoutingIntentKind,
        priority: QueuePriority,
        evidence: Vec<AdmissionEnvelope>,
    ) -> Result<Self, &'static str> {
        let work_item_key = work_item_key.into();
        let workflow_context = workflow_context.into();
        if work_item_key.trim().is_empty() {
            return Err("queueable_work_item work_item_key must not be empty");
        }
        if workflow_context.trim().is_empty() {
            return Err("queueable_work_item workflow_context must not be empty");
        }
        if evidence.is_empty() {
            return Err("queueable_work_item evidence must not be empty");
        }
        Ok(Self {
            work_item_key,
            workflow_context,
            intent,
            priority,
            evidence,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationState {
    Queued,
    CommandIssued,
    HeldForReview,
    Escalated,
    Refused,
    RecomputeTriggered,
    FallbackRouted,
    Suppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionCommandKind {
    PromoteForExecution,
    HoldForReview,
    Escalate,
    RefuseExecution,
    TriggerRecompute,
    RouteToFallback,
    SuppressOrdinaryRouting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingReason {
    AdmissionRejected,
    EvidenceReady,
    ControlReviewRequired,
    ControlDispositionRefused,
    CompatibilityGateBlocking,
    CompatibilityEvidenceMissing,
    ReleaseReadinessBlocked,
    ReleaseReadinessNeedsReview,
    ReleaseReadinessMissing,
    CacheRequiresRecompute,
    CacheBlocked,
    CacheRetired,
    BackendAdmissibilityMissing,
    BackendInadmissible,
    BackendRuntimeNeedsParity,
    CudaPromotionEvidenceMissing,
    CudaPromotionEligible,
    CudaBackendInadmissible,
    CudaWorkloadIneligible,
    CudaLayoutReviewRequired,
    CudaLayoutBreaking,
    CudaCanonicalizationRequired,
    CudaNonCanonicalizable,
    CudaParityWithinBudget,
    CudaParityOverBudget,
    AcceleratedRouteRequiresTypedEvidence,
    FallbackRequested,
    SuppressionRequested,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    pub decision_key: String,
    pub intent: RoutingIntentKind,
    pub command_kind: ExecutionCommandKind,
    pub reasons: Vec<RoutingReason>,
    pub evidence_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionCommand {
    pub command_key: String,
    pub workflow_context: String,
    pub command_kind: ExecutionCommandKind,
    pub evidence_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingExplanationPayload {
    pub explanation_key: String,
    pub summary: String,
    pub admitted_evidence_keys: Vec<String>,
    pub rejected_evidence_keys: Vec<String>,
    pub compute_reason_ids: Vec<String>,
    pub control_reason_ids: Vec<String>,
    pub audit_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationAuditRecord {
    pub audit_key: String,
    pub work_item_key: String,
    pub state: OrchestrationState,
    pub decision: RoutingDecision,
    pub command: ExecutionCommand,
    pub explanation_bundle: ExplanationBundle,
    pub routing_explanation: RoutingExplanationPayload,
}

pub fn route_work_item(
    work_item: &QueueableWorkItem,
) -> Result<OrchestrationAuditRecord, &'static str> {
    let trace = ControlTrace::new(
        format!("routing/{}", work_item.work_item_key),
        work_item.workflow_context.clone(),
        work_item.evidence.clone(),
    )?;
    let bundle = assemble_explanation_bundle(&trace)?;
    let decision = routing_decision_for(work_item, &bundle);
    let command = ExecutionCommand {
        command_key: format!("command/{}", decision.decision_key),
        workflow_context: work_item.workflow_context.clone(),
        command_kind: decision.command_kind,
        evidence_keys: decision.evidence_keys.clone(),
    };
    let state = orchestration_state_for(command.command_kind);
    let audit_key = format!("audit/{}", work_item.work_item_key);
    let routing_explanation = routing_explanation_for(work_item, &bundle, &decision, &audit_key);

    Ok(OrchestrationAuditRecord {
        audit_key,
        work_item_key: work_item.work_item_key.clone(),
        state,
        decision,
        command,
        explanation_bundle: bundle,
        routing_explanation,
    })
}

fn routing_decision_for(
    work_item: &QueueableWorkItem,
    bundle: &ExplanationBundle,
) -> RoutingDecision {
    let evidence_keys = work_item
        .evidence
        .iter()
        .map(|envelope| envelope.evidence_key.clone())
        .collect::<Vec<_>>();
    let mut reasons = Vec::new();

    let command_kind = if bundle
        .admission_records
        .iter()
        .any(|record| record.outcome == AdmissionOutcome::Rejected)
    {
        reasons.push(RoutingReason::AdmissionRejected);
        ExecutionCommandKind::RefuseExecution
    } else if has_release_block(&work_item.evidence) || has_compatibility_block(&work_item.evidence)
    {
        if has_release_block(&work_item.evidence) {
            reasons.push(RoutingReason::ReleaseReadinessBlocked);
        }
        if has_compatibility_block(&work_item.evidence) {
            reasons.push(RoutingReason::CompatibilityGateBlocking);
        }
        ExecutionCommandKind::Escalate
    } else if has_cache_retired(&work_item.evidence) {
        reasons.push(RoutingReason::CacheRetired);
        ExecutionCommandKind::RefuseExecution
    } else if has_cache_blocked(&work_item.evidence) {
        reasons.push(RoutingReason::CacheBlocked);
        ExecutionCommandKind::HoldForReview
    } else if work_item.intent != RoutingIntentKind::AcceleratedExecution
        && has_backend_inadmissible(&work_item.evidence)
    {
        reasons.push(RoutingReason::BackendInadmissible);
        ExecutionCommandKind::RefuseExecution
    } else if work_item.intent == RoutingIntentKind::AcceleratedExecution
        && has_release_needs_review(&work_item.evidence)
    {
        reasons.push(RoutingReason::ReleaseReadinessNeedsReview);
        ExecutionCommandKind::HoldForReview
    } else if work_item.intent == RoutingIntentKind::AcceleratedExecution
        && !has_release_ready(&work_item.evidence)
    {
        reasons.push(RoutingReason::AcceleratedRouteRequiresTypedEvidence);
        reasons.push(RoutingReason::ReleaseReadinessMissing);
        ExecutionCommandKind::HoldForReview
    } else if work_item.intent == RoutingIntentKind::AcceleratedExecution
        && !has_compatibility_clean(&work_item.evidence)
    {
        reasons.push(RoutingReason::AcceleratedRouteRequiresTypedEvidence);
        reasons.push(RoutingReason::CompatibilityEvidenceMissing);
        ExecutionCommandKind::HoldForReview
    } else if work_item.intent == RoutingIntentKind::AcceleratedExecution {
        match cuda_promotion_summary(&work_item.evidence) {
            Some(summary) => command_from_cuda_promotion(summary, &mut reasons),
            None => {
                reasons.push(RoutingReason::AcceleratedRouteRequiresTypedEvidence);
                reasons.push(RoutingReason::CudaPromotionEvidenceMissing);
                ExecutionCommandKind::HoldForReview
            }
        }
    } else if has_cache_recompute(&work_item.evidence)
        || work_item.intent == RoutingIntentKind::Recompute
    {
        reasons.push(RoutingReason::CacheRequiresRecompute);
        ExecutionCommandKind::TriggerRecompute
    } else {
        let disposition = routing_disposition_for_intent(work_item.intent, bundle);
        command_from_intent_or_disposition(work_item.intent, disposition, &mut reasons)
    };

    if reasons.is_empty() {
        reasons.push(RoutingReason::EvidenceReady);
    }

    RoutingDecision {
        decision_key: format!("decision/{}", work_item.work_item_key),
        intent: work_item.intent,
        command_kind,
        reasons,
        evidence_keys,
    }
}

fn command_from_cuda_promotion(
    summary: &CudaBackendPromotionSummary,
    reasons: &mut Vec<RoutingReason>,
) -> ExecutionCommandKind {
    let reason = cuda_promotion_routing_reason(summary.promotion_reason);
    reasons.push(reason);

    match summary.promotion_posture {
        CudaPromotionPosture::Promote => ExecutionCommandKind::PromoteForExecution,
        CudaPromotionPosture::Hold | CudaPromotionPosture::Degrade => {
            ExecutionCommandKind::HoldForReview
        }
        CudaPromotionPosture::Fallback => ExecutionCommandKind::RouteToFallback,
    }
}

fn command_from_intent_or_disposition(
    intent: RoutingIntentKind,
    disposition: Disposition,
    reasons: &mut Vec<RoutingReason>,
) -> ExecutionCommandKind {
    match intent {
        RoutingIntentKind::FallbackExecution => {
            reasons.push(RoutingReason::FallbackRequested);
            ExecutionCommandKind::RouteToFallback
        }
        RoutingIntentKind::Suppress => {
            reasons.push(RoutingReason::SuppressionRequested);
            ExecutionCommandKind::SuppressOrdinaryRouting
        }
        RoutingIntentKind::Review => {
            reasons.push(RoutingReason::ControlReviewRequired);
            ExecutionCommandKind::HoldForReview
        }
        RoutingIntentKind::NormalExecution
        | RoutingIntentKind::AcceleratedExecution
        | RoutingIntentKind::Recompute => match disposition {
            Disposition::Promote => ExecutionCommandKind::PromoteForExecution,
            Disposition::Fallback => {
                reasons.push(RoutingReason::FallbackRequested);
                ExecutionCommandKind::RouteToFallback
            }
            Disposition::Refuse => {
                reasons.push(RoutingReason::ControlDispositionRefused);
                ExecutionCommandKind::RefuseExecution
            }
            Disposition::Suppress => {
                reasons.push(RoutingReason::SuppressionRequested);
                ExecutionCommandKind::SuppressOrdinaryRouting
            }
            Disposition::Degrade | Disposition::HoldForReview => {
                reasons.push(RoutingReason::ControlReviewRequired);
                ExecutionCommandKind::HoldForReview
            }
            Disposition::Escalate => {
                reasons.push(RoutingReason::ControlReviewRequired);
                ExecutionCommandKind::Escalate
            }
        },
    }
}

fn routing_disposition_for_intent(
    intent: RoutingIntentKind,
    bundle: &ExplanationBundle,
) -> Disposition {
    if intent != RoutingIntentKind::AcceleratedExecution
        && only_backend_runtime_review_blocks(bundle)
    {
        Disposition::Promote
    } else {
        bundle.disposition
    }
}

fn only_backend_runtime_review_blocks(bundle: &ExplanationBundle) -> bool {
    let mut saw_backend_runtime_review = false;

    for record in &bundle.governance_records {
        if record
            .reasons
            .contains(&GovernanceReason::BackendRuntimeNeedsParity)
        {
            saw_backend_runtime_review = true;
            if record.reasons.len() == 1 && record.disposition == Disposition::HoldForReview {
                continue;
            }
        }
        if record.disposition != Disposition::Promote {
            return false;
        }
    }

    saw_backend_runtime_review
}

fn routing_explanation_for(
    work_item: &QueueableWorkItem,
    bundle: &ExplanationBundle,
    decision: &RoutingDecision,
    audit_key: &str,
) -> RoutingExplanationPayload {
    let admitted_evidence_keys = bundle
        .admission_records
        .iter()
        .filter(|record| record.outcome == AdmissionOutcome::Admitted)
        .map(|record| record.evidence_key.clone())
        .collect::<Vec<_>>();
    let rejected_evidence_keys = bundle
        .admission_records
        .iter()
        .filter(|record| record.outcome == AdmissionOutcome::Rejected)
        .map(|record| record.evidence_key.clone())
        .collect::<Vec<_>>();
    let compute_reason_ids = collect_compute_reason_ids(&work_item.evidence);
    let control_reason_ids = decision
        .reasons
        .iter()
        .copied()
        .map(routing_reason_id)
        .chain(
            bundle
                .governance_records
                .iter()
                .flat_map(|record| record.reasons.iter().copied().map(governance_reason_id)),
        )
        .map(str::to_string)
        .collect::<Vec<_>>();

    RoutingExplanationPayload {
        explanation_key: format!("routing_explanation/{}", decision.decision_key),
        summary: format!(
            "routing command={} evidence_count={} audit={}",
            execution_command_kind_id(decision.command_kind),
            work_item.evidence.len(),
            audit_key
        ),
        admitted_evidence_keys,
        rejected_evidence_keys,
        compute_reason_ids,
        control_reason_ids,
        audit_ref: audit_key.to_string(),
    }
}

fn collect_compute_reason_ids(evidence: &[AdmissionEnvelope]) -> Vec<String> {
    let mut ids = Vec::new();
    for envelope in evidence {
        match &envelope.summary {
            ComputeEvidenceSummary::PluginCompatibility(summary) => {
                ids.push(plugin_reason_id(summary.reason).to_string());
                ids.push(match summary.compatibility {
                    PluginCompatibility::Compatible => "plugin_compatible".to_string(),
                    PluginCompatibility::Incompatible => "plugin_incompatible".to_string(),
                });
            }
            ComputeEvidenceSummary::CachePolicy(summary) => {
                ids.push(cache_lifecycle_id(summary.lifecycle_state).to_string());
                ids.push(cache_reuse_id(summary.reuse_admissibility).to_string());
                ids.push(cache_coherence_id(summary.coherence_posture).to_string());
                if let Some(reason) = summary.recompute_reason {
                    ids.push(recompute_reason_id(reason).to_string());
                }
                if let Some(reason) = summary.blocked_reason {
                    ids.push(cache_blocked_reason_id(reason).to_string());
                }
            }
            ComputeEvidenceSummary::ContractCompatibilityGate(summary) => {
                ids.push(compatibility_classification_id(summary.classification).to_string());
                ids.extend(summary.reasons.iter().cloned());
            }
            ComputeEvidenceSummary::ReleaseReadiness(summary) => {
                ids.push(evidence_readiness_id(summary.readiness).to_string());
                ids.extend(summary.reasons.iter().cloned());
            }
            ComputeEvidenceSummary::BackendRuntimePosture(summary) => {
                ids.push(backend_memory_layout_id(summary.layout_posture).to_string());
                ids.push(precision_posture_id(summary.precision_posture).to_string());
                ids.push(backend_canonicalization_id(summary.canonicalization_posture).to_string());
                ids.push(backend_parity_oracle_id(summary.parity_oracle).to_string());
            }
            ComputeEvidenceSummary::CudaBackendPromotion(summary) => {
                ids.push(backend_admissibility_id(summary.backend_admissibility).to_string());
                ids.push(backend_runtime_track_id(summary.runtime_track).to_string());
                ids.push(backend_layout_version_id(summary.layout_version).to_string());
                ids.push(host_device_transfer_semantics_id(summary.transfer_semantics).to_string());
                ids.push(backend_precision_mode_id(summary.precision_mode).to_string());
                ids.push(backend_packing_policy_id(summary.packing_policy).to_string());
                ids.push(summary.canonicalization_boundary.clone());
                ids.push(backend_layout_compatibility_id(summary.layout_compatibility).to_string());
                ids.push(cuda_parity_budget_id(summary.parity_budget));
                ids.push(format!(
                    "observed_delta_units:{}",
                    summary.observed_delta_units
                ));
                ids.push(cuda_canonicalization_posture_id(summary.canonicalization).to_string());
                ids.push(cuda_workload_eligibility_id(summary.workload_eligibility).to_string());
                ids.push(
                    cuda_workload_eligibility_reason_id(summary.eligibility_reason).to_string(),
                );
                ids.push(cuda_parity_status_id(summary.parity_status).to_string());
                ids.push(cuda_promotion_posture_id(summary.promotion_posture).to_string());
                ids.push(cuda_promotion_reason_id(summary.promotion_reason).to_string());
            }
            ComputeEvidenceSummary::BackendAdmissibility {
                admissibility,
                reason,
            } => {
                ids.push(backend_admissibility_id(*admissibility).to_string());
                if !reason.trim().is_empty() {
                    ids.push(reason.clone());
                }
            }
            ComputeEvidenceSummary::GenericArtifact { artifact_family } => {
                ids.push(format!("generic_artifact:{artifact_family}"));
            }
        }
    }
    ids
}

fn orchestration_state_for(command: ExecutionCommandKind) -> OrchestrationState {
    match command {
        ExecutionCommandKind::PromoteForExecution => OrchestrationState::CommandIssued,
        ExecutionCommandKind::HoldForReview => OrchestrationState::HeldForReview,
        ExecutionCommandKind::Escalate => OrchestrationState::Escalated,
        ExecutionCommandKind::RefuseExecution => OrchestrationState::Refused,
        ExecutionCommandKind::TriggerRecompute => OrchestrationState::RecomputeTriggered,
        ExecutionCommandKind::RouteToFallback => OrchestrationState::FallbackRouted,
        ExecutionCommandKind::SuppressOrdinaryRouting => OrchestrationState::Suppressed,
    }
}

fn has_release_block(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::ReleaseReadiness(ref summary)
                if summary.readiness == EvidenceReadiness::Blocked
        )
    })
}

fn has_release_ready(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::ReleaseReadiness(ref summary)
                if summary.readiness == EvidenceReadiness::Ready
        )
    })
}

fn has_release_needs_review(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::ReleaseReadiness(ref summary)
                if summary.readiness == EvidenceReadiness::NeedsReview
        )
    })
}

fn has_compatibility_block(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::ContractCompatibilityGate(ref summary)
                if summary.gate_blocking
        )
    })
}

fn has_compatibility_clean(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::ContractCompatibilityGate(ref summary)
                if !summary.gate_blocking
                    && summary.classification == CompatibilityClassification::InternalOnly
        )
    })
}

fn has_cache_retired(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::CachePolicy(ref summary)
                if summary.lifecycle_state == CacheLifecycleState::Retired
        )
    })
}

fn has_cache_blocked(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::CachePolicy(ref summary) if is_cache_blocked(summary)
        )
    })
}

fn has_cache_recompute(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::CachePolicy(ref summary)
                if !is_cache_blocked(summary)
                    && (summary.recompute_reason.is_some()
                        || summary.reuse_admissibility == CacheReuseAdmissibility::ReuseRefused)
        )
    })
}

fn is_cache_blocked(summary: &CachePolicySummary) -> bool {
    summary.lifecycle_state == CacheLifecycleState::BlockedDependencyMissing
        || summary.blocked_reason == Some(CacheBlockedReason::DependencyMissing)
        || summary.coherence_posture == CacheCoherencePosture::DependencyMissing
}

fn has_backend_inadmissible(evidence: &[AdmissionEnvelope]) -> bool {
    evidence.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::BackendAdmissibility {
                admissibility: BackendAdmissibility::Inadmissible,
                ..
            }
        )
    })
}

fn cuda_promotion_summary(evidence: &[AdmissionEnvelope]) -> Option<&CudaBackendPromotionSummary> {
    evidence.iter().find_map(|envelope| {
        if let ComputeEvidenceSummary::CudaBackendPromotion(summary) = &envelope.summary {
            Some(summary)
        } else {
            None
        }
    })
}

fn cuda_promotion_routing_reason(reason: CudaPromotionReason) -> RoutingReason {
    match reason {
        CudaPromotionReason::PromotionEligible => RoutingReason::CudaPromotionEligible,
        CudaPromotionReason::ParityWithinBudget => RoutingReason::CudaParityWithinBudget,
        CudaPromotionReason::BackendInadmissible => RoutingReason::CudaBackendInadmissible,
        CudaPromotionReason::WorkloadIneligible => RoutingReason::CudaWorkloadIneligible,
        CudaPromotionReason::LayoutReviewRequired => RoutingReason::CudaLayoutReviewRequired,
        CudaPromotionReason::LayoutBreaking => RoutingReason::CudaLayoutBreaking,
        CudaPromotionReason::CanonicalizationRequired => {
            RoutingReason::CudaCanonicalizationRequired
        }
        CudaPromotionReason::NonCanonicalizable => RoutingReason::CudaNonCanonicalizable,
        CudaPromotionReason::ParityOverBudget => RoutingReason::CudaParityOverBudget,
    }
}

pub fn routing_reason_id(reason: RoutingReason) -> &'static str {
    match reason {
        RoutingReason::AdmissionRejected => "admission_rejected",
        RoutingReason::EvidenceReady => "evidence_ready",
        RoutingReason::ControlReviewRequired => "control_review_required",
        RoutingReason::ControlDispositionRefused => "control_disposition_refused",
        RoutingReason::CompatibilityGateBlocking => "compatibility_gate_blocking",
        RoutingReason::CompatibilityEvidenceMissing => "compatibility_evidence_missing",
        RoutingReason::ReleaseReadinessBlocked => "release_readiness_blocked",
        RoutingReason::ReleaseReadinessNeedsReview => "release_readiness_needs_review",
        RoutingReason::ReleaseReadinessMissing => "release_readiness_missing",
        RoutingReason::CacheRequiresRecompute => "cache_requires_recompute",
        RoutingReason::CacheBlocked => "cache_blocked",
        RoutingReason::CacheRetired => "cache_retired",
        RoutingReason::BackendAdmissibilityMissing => "backend_admissibility_missing",
        RoutingReason::BackendInadmissible => "backend_inadmissible",
        RoutingReason::BackendRuntimeNeedsParity => "backend_runtime_needs_parity",
        RoutingReason::CudaPromotionEvidenceMissing => "cuda_promotion_evidence_missing",
        RoutingReason::CudaPromotionEligible => "cuda_promotion_eligible",
        RoutingReason::CudaBackendInadmissible => "cuda_backend_inadmissible",
        RoutingReason::CudaWorkloadIneligible => "cuda_workload_ineligible",
        RoutingReason::CudaLayoutReviewRequired => "cuda_layout_review_required",
        RoutingReason::CudaLayoutBreaking => "cuda_layout_breaking",
        RoutingReason::CudaCanonicalizationRequired => "cuda_canonicalization_required",
        RoutingReason::CudaNonCanonicalizable => "cuda_non_canonicalizable",
        RoutingReason::CudaParityWithinBudget => "cuda_parity_within_budget",
        RoutingReason::CudaParityOverBudget => "cuda_parity_over_budget",
        RoutingReason::AcceleratedRouteRequiresTypedEvidence => {
            "accelerated_route_requires_typed_evidence"
        }
        RoutingReason::FallbackRequested => "fallback_requested",
        RoutingReason::SuppressionRequested => "suppression_requested",
    }
}

pub fn execution_command_kind_id(kind: ExecutionCommandKind) -> &'static str {
    match kind {
        ExecutionCommandKind::PromoteForExecution => "promote_for_execution",
        ExecutionCommandKind::HoldForReview => "hold_for_review",
        ExecutionCommandKind::Escalate => "escalate",
        ExecutionCommandKind::RefuseExecution => "refuse_execution",
        ExecutionCommandKind::TriggerRecompute => "trigger_recompute",
        ExecutionCommandKind::RouteToFallback => "route_to_fallback",
        ExecutionCommandKind::SuppressOrdinaryRouting => "suppress_ordinary_routing",
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

fn plugin_reason_id(reason: PluginCompatibilityReason) -> &'static str {
    match reason {
        PluginCompatibilityReason::DeclaredCompatible => "declared_compatible",
        PluginCompatibilityReason::PluginKeyMismatch => "plugin_key_mismatch",
        PluginCompatibilityReason::BoundaryKindMismatch => "boundary_kind_mismatch",
        PluginCompatibilityReason::OperationKindMismatch => "operation_kind_mismatch",
        PluginCompatibilityReason::BackendCapabilityInadmissible => {
            "backend_capability_inadmissible"
        }
    }
}

fn cache_lifecycle_id(state: CacheLifecycleState) -> &'static str {
    match state {
        CacheLifecycleState::Fresh => "fresh",
        CacheLifecycleState::Stale => "stale",
        CacheLifecycleState::Invalid => "invalid",
        CacheLifecycleState::Retired => "retired",
        CacheLifecycleState::BlockedDependencyMissing => "blocked_dependency_missing",
    }
}

fn cache_reuse_id(admissibility: CacheReuseAdmissibility) -> &'static str {
    match admissibility {
        CacheReuseAdmissibility::ReuseAdmissible => "reuse_admissible",
        CacheReuseAdmissibility::ReuseRefused => "reuse_refused",
    }
}

fn cache_coherence_id(posture: CacheCoherencePosture) -> &'static str {
    match posture {
        CacheCoherencePosture::Coherent => "coherent",
        CacheCoherencePosture::DependencyMissing => "dependency_missing",
        CacheCoherencePosture::DependencyContentDrift => "dependency_content_drift",
    }
}

fn recompute_reason_id(reason: RecomputeReason) -> &'static str {
    match reason {
        RecomputeReason::DirectArtifactChanged => "direct_artifact_changed",
        RecomputeReason::UpstreamDependencyChanged => "upstream_dependency_changed",
        RecomputeReason::DependencyContentDrift => "dependency_content_drift",
        RecomputeReason::ArtifactRetired => "artifact_retired",
    }
}

fn cache_blocked_reason_id(reason: CacheBlockedReason) -> &'static str {
    match reason {
        CacheBlockedReason::DependencyMissing => "dependency_missing",
    }
}

fn compatibility_classification_id(classification: CompatibilityClassification) -> &'static str {
    match classification {
        CompatibilityClassification::Additive => "additive",
        CompatibilityClassification::CompatibleTightening => "compatible_tightening",
        CompatibilityClassification::Breaking => "breaking",
        CompatibilityClassification::InternalOnly => "internal_only",
    }
}

fn evidence_readiness_id(readiness: EvidenceReadiness) -> &'static str {
    match readiness {
        EvidenceReadiness::Ready => "ready",
        EvidenceReadiness::NeedsReview => "needs_review",
        EvidenceReadiness::Blocked => "blocked",
        EvidenceReadiness::NotApplicable => "not_applicable",
    }
}

fn backend_memory_layout_id(posture: BackendMemoryLayoutPosture) -> &'static str {
    match posture {
        BackendMemoryLayoutPosture::HostCanonical => "host_canonical",
        BackendMemoryLayoutPosture::DeviceShadowVersioned => "device_shadow_versioned",
        BackendMemoryLayoutPosture::VersionMismatch => "version_mismatch",
        BackendMemoryLayoutPosture::Unknown => "unknown",
    }
}

fn precision_posture_id(posture: PrecisionPosture) -> &'static str {
    match posture {
        PrecisionPosture::DeterministicReference => "deterministic_reference",
        PrecisionPosture::ExplicitPolicy => "explicit_policy",
        PrecisionPosture::Mismatch => "mismatch",
        PrecisionPosture::Unknown => "unknown",
    }
}

fn backend_canonicalization_id(posture: BackendCanonicalizationPosture) -> &'static str {
    match posture {
        BackendCanonicalizationPosture::BackendIndependent => "backend_independent",
        BackendCanonicalizationPosture::BackendLocalOnly => "backend_local_only",
        BackendCanonicalizationPosture::Unknown => "unknown",
    }
}

fn backend_parity_oracle_id(oracle: BackendParityOracle) -> &'static str {
    match oracle {
        BackendParityOracle::CpuReference => "cpu_reference",
        BackendParityOracle::Missing => "missing",
        BackendParityOracle::Unknown => "unknown",
    }
}

fn backend_admissibility_id(admissibility: BackendAdmissibility) -> &'static str {
    match admissibility {
        BackendAdmissibility::Admissible => "admissible",
        BackendAdmissibility::Inadmissible => "inadmissible",
    }
}

fn backend_runtime_track_id(track: BackendRuntimeTrack) -> &'static str {
    match track {
        BackendRuntimeTrack::CpuReference => "cpu_reference",
        BackendRuntimeTrack::CudaOptimized => "cuda_optimized",
    }
}

fn backend_layout_version_id(version: BackendLayoutVersion) -> &'static str {
    match version {
        BackendLayoutVersion::HostCanonicalV0 => "host_canonical_v0",
        BackendLayoutVersion::CudaDeviceTensorV0 => "cuda_device_tensor_v0",
    }
}

fn host_device_transfer_semantics_id(semantics: HostDeviceTransferSemantics) -> &'static str {
    match semantics {
        HostDeviceTransferSemantics::HostLocal => "host_local",
        HostDeviceTransferSemantics::HostDeviceRoundTrip => "host_device_round_trip",
        HostDeviceTransferSemantics::DeviceOnlyNoCanonicalReturn => {
            "device_only_no_canonical_return"
        }
    }
}

fn backend_precision_mode_id(mode: BackendPrecisionMode) -> &'static str {
    match mode {
        BackendPrecisionMode::DeterministicReference => "deterministic_reference",
        BackendPrecisionMode::Float32Deterministic => "float32_deterministic",
        BackendPrecisionMode::Float64Deterministic => "float64_deterministic",
        BackendPrecisionMode::MixedPrecisionExplicit => "mixed_precision_explicit",
    }
}

fn backend_packing_policy_id(policy: BackendPackingPolicy) -> &'static str {
    match policy {
        BackendPackingPolicy::CanonicalHost => "canonical_host",
        BackendPackingPolicy::DeviceLocalContiguous => "device_local_contiguous",
        BackendPackingPolicy::ExplicitTensorPacked => "explicit_tensor_packed",
    }
}

fn backend_layout_compatibility_id(compatibility: BackendLayoutCompatibility) -> &'static str {
    match compatibility {
        BackendLayoutCompatibility::Compatible => "compatible",
        BackendLayoutCompatibility::Canonicalizable => "canonicalizable",
        BackendLayoutCompatibility::ReviewRequired => "review_required",
        BackendLayoutCompatibility::Breaking => "breaking",
    }
}

fn cuda_parity_budget_id(budget: CudaParityBudget) -> String {
    match budget {
        CudaParityBudget::Exact => "exact".to_string(),
        CudaParityBudget::BoundedUnits { max_delta_units } => {
            format!("bounded_units:{max_delta_units}")
        }
    }
}

fn cuda_canonicalization_posture_id(posture: CudaCanonicalizationPosture) -> &'static str {
    match posture {
        CudaCanonicalizationPosture::Canonicalized => "canonicalized",
        CudaCanonicalizationPosture::RematerializationRequired => "rematerialization_required",
        CudaCanonicalizationPosture::NonCanonicalizable => "non_canonicalizable",
    }
}

fn cuda_workload_eligibility_id(eligibility: CudaWorkloadEligibility) -> &'static str {
    match eligibility {
        CudaWorkloadEligibility::Eligible => "eligible",
        CudaWorkloadEligibility::Ineligible => "ineligible",
    }
}

fn cuda_workload_eligibility_reason_id(reason: CudaWorkloadEligibilityReason) -> &'static str {
    match reason {
        CudaWorkloadEligibilityReason::EngineWorkloadEligible => "engine_workload_eligible",
        CudaWorkloadEligibilityReason::BackendInadmissible => "backend_inadmissible",
        CudaWorkloadEligibilityReason::RuntimeTrackNotCuda => "runtime_track_not_cuda",
        CudaWorkloadEligibilityReason::WorkloadNotCudaEligible => "workload_not_cuda_eligible",
    }
}

fn cuda_parity_status_id(status: CudaParityStatus) -> &'static str {
    match status {
        CudaParityStatus::ParityClean => "parity_clean",
        CudaParityStatus::ParityWithinBudget => "parity_within_budget",
        CudaParityStatus::ParityOverBudget => "parity_over_budget",
    }
}

fn cuda_promotion_posture_id(posture: CudaPromotionPosture) -> &'static str {
    match posture {
        CudaPromotionPosture::Promote => "promote",
        CudaPromotionPosture::Hold => "hold",
        CudaPromotionPosture::Degrade => "degrade",
        CudaPromotionPosture::Fallback => "fallback",
    }
}

fn cuda_promotion_reason_id(reason: CudaPromotionReason) -> &'static str {
    match reason {
        CudaPromotionReason::PromotionEligible => "promotion_eligible",
        CudaPromotionReason::ParityWithinBudget => "parity_within_budget",
        CudaPromotionReason::BackendInadmissible => "backend_inadmissible",
        CudaPromotionReason::WorkloadIneligible => "workload_ineligible",
        CudaPromotionReason::LayoutReviewRequired => "layout_review_required",
        CudaPromotionReason::LayoutBreaking => "layout_breaking",
        CudaPromotionReason::CanonicalizationRequired => "canonicalization_required",
        CudaPromotionReason::NonCanonicalizable => "non_canonicalizable",
        CudaPromotionReason::ParityOverBudget => "parity_over_budget",
    }
}
