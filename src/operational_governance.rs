use crate::admission::AdmissionEnvelope;
use crate::evidence::{
    BackendAdmissibility, BackendCanonicalizationPosture, BackendMemoryLayoutPosture,
    BackendParityOracle, CompatibilityClassification, ComputeEvidenceSummary, CudaPromotionPosture,
    EvidenceReadiness, PrecisionPosture,
};
use crate::orchestration::OrchestrationAuditRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyCompatibilityPosture {
    Compatible,
    ReviewRequired,
    Breaking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernancePolicySet {
    pub policy_set_key: String,
    pub policy_version: String,
    pub compatibility: PolicyCompatibilityPosture,
    pub compatibility_reasons: Vec<String>,
}

impl GovernancePolicySet {
    pub fn new(
        policy_set_key: impl Into<String>,
        policy_version: impl Into<String>,
        compatibility: PolicyCompatibilityPosture,
        compatibility_reasons: Vec<String>,
    ) -> Result<Self, &'static str> {
        let policy_set_key = policy_set_key.into();
        let policy_version = policy_version.into();
        if policy_set_key.trim().is_empty() {
            return Err("governance_policy_set policy_set_key must not be empty");
        }
        if policy_version.trim().is_empty() {
            return Err("governance_policy_set policy_version must not be empty");
        }
        if compatibility != PolicyCompatibilityPosture::Compatible
            && compatibility_reasons
                .iter()
                .all(|reason| reason.trim().is_empty())
        {
            return Err(
                "governance_policy_set non-compatible posture requires compatibility reason",
            );
        }
        Ok(Self {
            policy_set_key,
            policy_version,
            compatibility,
            compatibility_reasons,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlReleaseReadinessStatus {
    Ready,
    NeedsReview,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlReleaseReadinessReason {
    PolicySetPresent,
    PolicySetMissing,
    PolicyCompatible,
    PolicyReviewRequired,
    PolicyBreaking,
    AuditRequirementsSatisfied,
    AuditRequirementsMissing,
    GovernanceTraceCorpusGreen,
    GovernanceTraceCorpusNeedsReview,
    WorkflowTransitionGuardsPresent,
    WorkflowTransitionGuardsMissing,
    UpstreamComputeEvidenceConsumable,
    UpstreamComputeEvidenceMissing,
    RollbackDoctrinePresent,
    RollbackDoctrineMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlReleaseReadinessInput {
    pub readiness_key: String,
    pub policy_set: Option<GovernancePolicySet>,
    pub audit_requirements_satisfied: bool,
    pub governance_trace_corpus_green: bool,
    pub workflow_transition_guards_present: bool,
    pub upstream_compute_evidence_consumable: bool,
    pub rollback_doctrine_present: bool,
    pub replay_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlReleaseReadinessReport {
    pub readiness_key: String,
    pub status: ControlReleaseReadinessStatus,
    pub reasons: Vec<ControlReleaseReadinessReason>,
    pub policy_version: Option<String>,
    pub replay_note: String,
}

pub fn control_release_readiness_report(
    input: ControlReleaseReadinessInput,
) -> Result<ControlReleaseReadinessReport, &'static str> {
    if input.readiness_key.trim().is_empty() {
        return Err("control_release_readiness readiness_key must not be empty");
    }
    if input.replay_note.trim().is_empty() {
        return Err("control_release_readiness replay_note must not be empty");
    }

    let mut reasons = Vec::new();
    let policy_version = match input.policy_set {
        Some(policy_set) => {
            reasons.push(ControlReleaseReadinessReason::PolicySetPresent);
            match policy_set.compatibility {
                PolicyCompatibilityPosture::Compatible => {
                    reasons.push(ControlReleaseReadinessReason::PolicyCompatible);
                }
                PolicyCompatibilityPosture::ReviewRequired => {
                    reasons.push(ControlReleaseReadinessReason::PolicyReviewRequired);
                }
                PolicyCompatibilityPosture::Breaking => {
                    reasons.push(ControlReleaseReadinessReason::PolicyBreaking);
                }
            }
            Some(policy_set.policy_version)
        }
        None => {
            reasons.push(ControlReleaseReadinessReason::PolicySetMissing);
            None
        }
    };

    push_bool_reason(
        &mut reasons,
        input.audit_requirements_satisfied,
        ControlReleaseReadinessReason::AuditRequirementsSatisfied,
        ControlReleaseReadinessReason::AuditRequirementsMissing,
    );
    push_bool_reason(
        &mut reasons,
        input.governance_trace_corpus_green,
        ControlReleaseReadinessReason::GovernanceTraceCorpusGreen,
        ControlReleaseReadinessReason::GovernanceTraceCorpusNeedsReview,
    );
    push_bool_reason(
        &mut reasons,
        input.workflow_transition_guards_present,
        ControlReleaseReadinessReason::WorkflowTransitionGuardsPresent,
        ControlReleaseReadinessReason::WorkflowTransitionGuardsMissing,
    );
    push_bool_reason(
        &mut reasons,
        input.upstream_compute_evidence_consumable,
        ControlReleaseReadinessReason::UpstreamComputeEvidenceConsumable,
        ControlReleaseReadinessReason::UpstreamComputeEvidenceMissing,
    );
    push_bool_reason(
        &mut reasons,
        input.rollback_doctrine_present,
        ControlReleaseReadinessReason::RollbackDoctrinePresent,
        ControlReleaseReadinessReason::RollbackDoctrineMissing,
    );

    let status = control_release_readiness_status(&reasons);
    Ok(ControlReleaseReadinessReport {
        readiness_key: input.readiness_key,
        status,
        reasons,
        policy_version,
        replay_note: input.replay_note,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemReleasePosture {
    Promotable,
    HoldForReview,
    DegradedButGovernable,
    FallbackOnly,
    RollbackRequired,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceOperationalState {
    Promoted,
    Held,
    Degraded,
    FallbackOnly,
    Escalated,
    RollbackRequired,
    RollbackInEffect,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemReleaseGovernanceReason {
    ControlReadinessReady,
    ControlReadinessNeedsReview,
    ControlReadinessBlocked,
    ComputeCompatibilityClean,
    ComputeCompatibilityGateBlocking,
    ComputeReleaseReady,
    ComputeReleaseNeedsReview,
    ComputeReleaseBlocked,
    BackendRolloutReady,
    BackendRolloutHold,
    BackendRolloutDegraded,
    BackendRolloutFallbackOnly,
    IncidentOperationalHold,
    IncidentDegradedOperation,
    IncidentEscalationRequired,
    IncidentRollbackRequired,
    IncidentRollbackInEffect,
    IncidentBlocked,
    AuditComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceIncidentKind {
    CompatibilityIncident,
    ReleaseReadinessIncident,
    BackendLayoutIncident,
    AuditIncident,
    PolicyIncident,
    OperatorIncident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceIncidentResponse {
    OperationalHold,
    DegradedOperation,
    EscalationRequired,
    RollbackRequired,
    RollbackInEffect,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceIncident {
    pub incident_key: String,
    pub kind: GovernanceIncidentKind,
    pub response: GovernanceIncidentResponse,
    pub summary: String,
}

impl GovernanceIncident {
    pub fn new(
        incident_key: impl Into<String>,
        kind: GovernanceIncidentKind,
        response: GovernanceIncidentResponse,
        summary: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let incident_key = incident_key.into();
        let summary = summary.into();
        if incident_key.trim().is_empty() {
            return Err("governance_incident incident_key must not be empty");
        }
        if summary.trim().is_empty() {
            return Err("governance_incident summary must not be empty");
        }
        Ok(Self {
            incident_key,
            kind,
            response,
            summary,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemReleaseGovernanceInput {
    pub governance_key: String,
    pub control_readiness: ControlReleaseReadinessReport,
    pub compute_evidence: Vec<AdmissionEnvelope>,
    pub orchestration_audit: OrchestrationAuditRecord,
    pub incident: Option<GovernanceIncident>,
    pub replay_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemReleaseGovernanceRecord {
    pub governance_key: String,
    pub posture: SystemReleasePosture,
    pub operational_state: GovernanceOperationalState,
    pub reasons: Vec<SystemReleaseGovernanceReason>,
    pub evidence_keys: Vec<String>,
    pub audit_ref: String,
    pub explanation_ref: String,
    pub policy_version: Option<String>,
    pub replay_note: String,
}

pub fn evaluate_system_release_governance(
    input: SystemReleaseGovernanceInput,
) -> Result<SystemReleaseGovernanceRecord, &'static str> {
    if input.governance_key.trim().is_empty() {
        return Err("system_release_governance governance_key must not be empty");
    }
    if input.replay_note.trim().is_empty() {
        return Err("system_release_governance replay_note must not be empty");
    }
    if input.compute_evidence.is_empty() {
        return Err("system_release_governance compute_evidence must not be empty");
    }
    validate_audit_completeness(&input.compute_evidence, &input.orchestration_audit)?;

    let mut reasons = Vec::new();
    push_control_readiness_reason(&mut reasons, input.control_readiness.status);
    push_compute_evidence_reasons(&mut reasons, &input.compute_evidence);
    if let Some(incident) = &input.incident {
        reasons.push(incident_reason(incident.response));
    }
    reasons.push(SystemReleaseGovernanceReason::AuditComplete);

    let posture = system_release_posture(&reasons);
    let operational_state = operational_state_for(posture, input.incident.as_ref());
    let evidence_keys = input
        .compute_evidence
        .iter()
        .map(|envelope| envelope.evidence_key.clone())
        .collect::<Vec<_>>();

    Ok(SystemReleaseGovernanceRecord {
        governance_key: input.governance_key,
        posture,
        operational_state,
        reasons,
        evidence_keys,
        audit_ref: input.orchestration_audit.audit_key,
        explanation_ref: input
            .orchestration_audit
            .routing_explanation
            .explanation_key,
        policy_version: input.control_readiness.policy_version,
        replay_note: input.replay_note,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceTransitionGuardOutcome {
    Allowed,
    ReviewRequired,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceTransitionGuardReason {
    AuditComplete,
    AuditIncomplete,
    ExplanationPresent,
    ExplanationMissing,
    EvidencePresent,
    EvidenceMissing,
    PromotionAllowed,
    PromotionRequiresExplicitReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceTransitionRequest {
    pub from: SystemReleasePosture,
    pub to: SystemReleasePosture,
    pub audit_complete: bool,
    pub explanation_present: bool,
    pub evidence_keys: Vec<String>,
    pub explicit_review_recorded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceTransitionGuard {
    pub outcome: GovernanceTransitionGuardOutcome,
    pub reasons: Vec<GovernanceTransitionGuardReason>,
}

pub fn evaluate_governance_transition(
    request: GovernanceTransitionRequest,
) -> GovernanceTransitionGuard {
    let mut reasons = Vec::new();
    push_bool_reason(
        &mut reasons,
        request.audit_complete,
        GovernanceTransitionGuardReason::AuditComplete,
        GovernanceTransitionGuardReason::AuditIncomplete,
    );
    push_bool_reason(
        &mut reasons,
        request.explanation_present,
        GovernanceTransitionGuardReason::ExplanationPresent,
        GovernanceTransitionGuardReason::ExplanationMissing,
    );
    push_bool_reason(
        &mut reasons,
        !request.evidence_keys.is_empty(),
        GovernanceTransitionGuardReason::EvidencePresent,
        GovernanceTransitionGuardReason::EvidenceMissing,
    );

    let mut outcome = if reasons.iter().any(|reason| {
        matches!(
            reason,
            GovernanceTransitionGuardReason::AuditIncomplete
                | GovernanceTransitionGuardReason::ExplanationMissing
                | GovernanceTransitionGuardReason::EvidenceMissing
        )
    }) {
        GovernanceTransitionGuardOutcome::Blocked
    } else {
        GovernanceTransitionGuardOutcome::Allowed
    };

    if request.to == SystemReleasePosture::Promotable
        && request.from != SystemReleasePosture::Promotable
    {
        if request.explicit_review_recorded {
            reasons.push(GovernanceTransitionGuardReason::PromotionAllowed);
        } else if outcome != GovernanceTransitionGuardOutcome::Blocked {
            reasons.push(GovernanceTransitionGuardReason::PromotionRequiresExplicitReview);
            outcome = GovernanceTransitionGuardOutcome::ReviewRequired;
        }
    }

    GovernanceTransitionGuard { outcome, reasons }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceTraceScenario {
    pub scenario_key: String,
    pub input: SystemReleaseGovernanceInput,
    pub expected_posture: SystemReleasePosture,
    pub expected_state: GovernanceOperationalState,
    pub required_reason: SystemReleaseGovernanceReason,
}

pub fn evaluate_governance_trace_scenario(
    scenario: GovernanceTraceScenario,
) -> Result<SystemReleaseGovernanceRecord, &'static str> {
    if scenario.scenario_key.trim().is_empty() {
        return Err("governance_trace_scenario scenario_key must not be empty");
    }
    let record = evaluate_system_release_governance(scenario.input)?;
    if record.posture != scenario.expected_posture {
        return Err("governance_trace_scenario posture did not match expectation");
    }
    if record.operational_state != scenario.expected_state {
        return Err("governance_trace_scenario state did not match expectation");
    }
    if !record.reasons.contains(&scenario.required_reason) {
        return Err("governance_trace_scenario required reason missing");
    }
    Ok(record)
}

fn push_bool_reason<T: Copy>(reasons: &mut Vec<T>, condition: bool, yes: T, no: T) {
    if condition {
        reasons.push(yes);
    } else {
        reasons.push(no);
    }
}

fn control_release_readiness_status(
    reasons: &[ControlReleaseReadinessReason],
) -> ControlReleaseReadinessStatus {
    if reasons.iter().any(|reason| {
        matches!(
            reason,
            ControlReleaseReadinessReason::PolicySetMissing
                | ControlReleaseReadinessReason::PolicyBreaking
                | ControlReleaseReadinessReason::AuditRequirementsMissing
                | ControlReleaseReadinessReason::WorkflowTransitionGuardsMissing
                | ControlReleaseReadinessReason::UpstreamComputeEvidenceMissing
                | ControlReleaseReadinessReason::RollbackDoctrineMissing
        )
    }) {
        ControlReleaseReadinessStatus::Blocked
    } else if reasons.iter().any(|reason| {
        matches!(
            reason,
            ControlReleaseReadinessReason::PolicyReviewRequired
                | ControlReleaseReadinessReason::GovernanceTraceCorpusNeedsReview
        )
    }) {
        ControlReleaseReadinessStatus::NeedsReview
    } else {
        ControlReleaseReadinessStatus::Ready
    }
}

fn push_control_readiness_reason(
    reasons: &mut Vec<SystemReleaseGovernanceReason>,
    status: ControlReleaseReadinessStatus,
) {
    reasons.push(match status {
        ControlReleaseReadinessStatus::Ready => {
            SystemReleaseGovernanceReason::ControlReadinessReady
        }
        ControlReleaseReadinessStatus::NeedsReview => {
            SystemReleaseGovernanceReason::ControlReadinessNeedsReview
        }
        ControlReleaseReadinessStatus::Blocked => {
            SystemReleaseGovernanceReason::ControlReadinessBlocked
        }
    });
}

fn push_compute_evidence_reasons(
    reasons: &mut Vec<SystemReleaseGovernanceReason>,
    evidence: &[AdmissionEnvelope],
) {
    for envelope in evidence {
        match &envelope.summary {
            ComputeEvidenceSummary::ContractCompatibilityGate(summary) => {
                if summary.gate_blocking
                    || summary.classification != CompatibilityClassification::InternalOnly
                {
                    reasons.push(SystemReleaseGovernanceReason::ComputeCompatibilityGateBlocking);
                } else {
                    reasons.push(SystemReleaseGovernanceReason::ComputeCompatibilityClean);
                }
            }
            ComputeEvidenceSummary::ReleaseReadiness(summary) => match summary.readiness {
                EvidenceReadiness::Ready => {
                    reasons.push(SystemReleaseGovernanceReason::ComputeReleaseReady);
                }
                EvidenceReadiness::NeedsReview => {
                    reasons.push(SystemReleaseGovernanceReason::ComputeReleaseNeedsReview);
                }
                EvidenceReadiness::Blocked | EvidenceReadiness::NotApplicable => {
                    reasons.push(SystemReleaseGovernanceReason::ComputeReleaseBlocked);
                }
            },
            ComputeEvidenceSummary::CudaBackendPromotion(summary) => {
                reasons.push(match summary.promotion_posture {
                    CudaPromotionPosture::Promote => {
                        SystemReleaseGovernanceReason::BackendRolloutReady
                    }
                    CudaPromotionPosture::Hold => SystemReleaseGovernanceReason::BackendRolloutHold,
                    CudaPromotionPosture::Degrade => {
                        SystemReleaseGovernanceReason::BackendRolloutDegraded
                    }
                    CudaPromotionPosture::Fallback => {
                        SystemReleaseGovernanceReason::BackendRolloutFallbackOnly
                    }
                });
            }
            ComputeEvidenceSummary::BackendAdmissibility {
                admissibility: BackendAdmissibility::Inadmissible,
                ..
            } => {
                reasons.push(SystemReleaseGovernanceReason::BackendRolloutFallbackOnly);
            }
            ComputeEvidenceSummary::BackendRuntimePosture(summary)
                if backend_runtime_requires_review(summary) =>
            {
                reasons.push(SystemReleaseGovernanceReason::BackendRolloutHold);
            }
            _ => {}
        }
    }
}

fn backend_runtime_requires_review(
    summary: &crate::evidence::BackendRuntimePostureSummary,
) -> bool {
    summary.layout_posture == BackendMemoryLayoutPosture::VersionMismatch
        || summary.precision_posture == PrecisionPosture::Mismatch
        || summary.canonicalization_posture != BackendCanonicalizationPosture::BackendIndependent
        || summary.parity_oracle != BackendParityOracle::CpuReference
}

fn incident_reason(response: GovernanceIncidentResponse) -> SystemReleaseGovernanceReason {
    match response {
        GovernanceIncidentResponse::OperationalHold => {
            SystemReleaseGovernanceReason::IncidentOperationalHold
        }
        GovernanceIncidentResponse::DegradedOperation => {
            SystemReleaseGovernanceReason::IncidentDegradedOperation
        }
        GovernanceIncidentResponse::EscalationRequired => {
            SystemReleaseGovernanceReason::IncidentEscalationRequired
        }
        GovernanceIncidentResponse::RollbackRequired => {
            SystemReleaseGovernanceReason::IncidentRollbackRequired
        }
        GovernanceIncidentResponse::RollbackInEffect => {
            SystemReleaseGovernanceReason::IncidentRollbackInEffect
        }
        GovernanceIncidentResponse::Blocked => SystemReleaseGovernanceReason::IncidentBlocked,
    }
}

fn system_release_posture(reasons: &[SystemReleaseGovernanceReason]) -> SystemReleasePosture {
    if has_any(
        reasons,
        &[
            SystemReleaseGovernanceReason::ControlReadinessBlocked,
            SystemReleaseGovernanceReason::ComputeCompatibilityGateBlocking,
            SystemReleaseGovernanceReason::ComputeReleaseBlocked,
            SystemReleaseGovernanceReason::IncidentBlocked,
        ],
    ) {
        SystemReleasePosture::Blocked
    } else if has_any(
        reasons,
        &[
            SystemReleaseGovernanceReason::IncidentRollbackRequired,
            SystemReleaseGovernanceReason::IncidentRollbackInEffect,
        ],
    ) {
        SystemReleasePosture::RollbackRequired
    } else if reasons.contains(&SystemReleaseGovernanceReason::BackendRolloutFallbackOnly) {
        SystemReleasePosture::FallbackOnly
    } else if has_any(
        reasons,
        &[
            SystemReleaseGovernanceReason::BackendRolloutDegraded,
            SystemReleaseGovernanceReason::IncidentDegradedOperation,
        ],
    ) {
        SystemReleasePosture::DegradedButGovernable
    } else if has_any(
        reasons,
        &[
            SystemReleaseGovernanceReason::ControlReadinessNeedsReview,
            SystemReleaseGovernanceReason::ComputeReleaseNeedsReview,
            SystemReleaseGovernanceReason::BackendRolloutHold,
            SystemReleaseGovernanceReason::IncidentOperationalHold,
            SystemReleaseGovernanceReason::IncidentEscalationRequired,
        ],
    ) {
        SystemReleasePosture::HoldForReview
    } else {
        SystemReleasePosture::Promotable
    }
}

fn operational_state_for(
    posture: SystemReleasePosture,
    incident: Option<&GovernanceIncident>,
) -> GovernanceOperationalState {
    if incident
        .map(|incident| incident.response == GovernanceIncidentResponse::RollbackInEffect)
        .unwrap_or(false)
    {
        return GovernanceOperationalState::RollbackInEffect;
    }
    if incident
        .map(|incident| incident.response == GovernanceIncidentResponse::EscalationRequired)
        .unwrap_or(false)
    {
        return GovernanceOperationalState::Escalated;
    }
    match posture {
        SystemReleasePosture::Promotable => GovernanceOperationalState::Promoted,
        SystemReleasePosture::HoldForReview => GovernanceOperationalState::Held,
        SystemReleasePosture::DegradedButGovernable => GovernanceOperationalState::Degraded,
        SystemReleasePosture::FallbackOnly => GovernanceOperationalState::FallbackOnly,
        SystemReleasePosture::RollbackRequired => GovernanceOperationalState::RollbackRequired,
        SystemReleasePosture::Blocked => GovernanceOperationalState::Blocked,
    }
}

fn has_any<T: PartialEq>(items: &[T], expected: &[T]) -> bool {
    expected.iter().any(|expected| items.contains(expected))
}

fn validate_audit_completeness(
    evidence: &[AdmissionEnvelope],
    audit: &OrchestrationAuditRecord,
) -> Result<(), &'static str> {
    if audit.audit_key.trim().is_empty() {
        return Err("system_release_governance audit_key must not be empty");
    }
    if audit.routing_explanation.explanation_key.trim().is_empty() {
        return Err("system_release_governance explanation_key must not be empty");
    }
    if audit.routing_explanation.audit_ref.trim().is_empty() {
        return Err("system_release_governance audit_ref must not be empty");
    }
    for envelope in evidence {
        if !audit
            .decision
            .evidence_keys
            .contains(&envelope.evidence_key)
        {
            return Err("system_release_governance audit must cover all evidence keys");
        }
    }
    Ok(())
}

pub fn policy_compatibility_posture_id(posture: PolicyCompatibilityPosture) -> &'static str {
    match posture {
        PolicyCompatibilityPosture::Compatible => "compatible",
        PolicyCompatibilityPosture::ReviewRequired => "review_required",
        PolicyCompatibilityPosture::Breaking => "breaking",
    }
}

pub fn control_release_readiness_status_id(status: ControlReleaseReadinessStatus) -> &'static str {
    match status {
        ControlReleaseReadinessStatus::Ready => "ready",
        ControlReleaseReadinessStatus::NeedsReview => "needs_review",
        ControlReleaseReadinessStatus::Blocked => "blocked",
    }
}

pub fn system_release_posture_id(posture: SystemReleasePosture) -> &'static str {
    match posture {
        SystemReleasePosture::Promotable => "promotable",
        SystemReleasePosture::HoldForReview => "hold_for_review",
        SystemReleasePosture::DegradedButGovernable => "degraded_but_governable",
        SystemReleasePosture::FallbackOnly => "fallback_only",
        SystemReleasePosture::RollbackRequired => "rollback_required",
        SystemReleasePosture::Blocked => "blocked",
    }
}

pub fn system_release_governance_reason_id(reason: SystemReleaseGovernanceReason) -> &'static str {
    match reason {
        SystemReleaseGovernanceReason::ControlReadinessReady => "control_readiness_ready",
        SystemReleaseGovernanceReason::ControlReadinessNeedsReview => {
            "control_readiness_needs_review"
        }
        SystemReleaseGovernanceReason::ControlReadinessBlocked => "control_readiness_blocked",
        SystemReleaseGovernanceReason::ComputeCompatibilityClean => "compute_compatibility_clean",
        SystemReleaseGovernanceReason::ComputeCompatibilityGateBlocking => {
            "compute_compatibility_gate_blocking"
        }
        SystemReleaseGovernanceReason::ComputeReleaseReady => "compute_release_ready",
        SystemReleaseGovernanceReason::ComputeReleaseNeedsReview => "compute_release_needs_review",
        SystemReleaseGovernanceReason::ComputeReleaseBlocked => "compute_release_blocked",
        SystemReleaseGovernanceReason::BackendRolloutReady => "backend_rollout_ready",
        SystemReleaseGovernanceReason::BackendRolloutHold => "backend_rollout_hold",
        SystemReleaseGovernanceReason::BackendRolloutDegraded => "backend_rollout_degraded",
        SystemReleaseGovernanceReason::BackendRolloutFallbackOnly => {
            "backend_rollout_fallback_only"
        }
        SystemReleaseGovernanceReason::IncidentOperationalHold => "incident_operational_hold",
        SystemReleaseGovernanceReason::IncidentDegradedOperation => "incident_degraded_operation",
        SystemReleaseGovernanceReason::IncidentEscalationRequired => "incident_escalation_required",
        SystemReleaseGovernanceReason::IncidentRollbackRequired => "incident_rollback_required",
        SystemReleaseGovernanceReason::IncidentRollbackInEffect => "incident_rollback_in_effect",
        SystemReleaseGovernanceReason::IncidentBlocked => "incident_blocked",
        SystemReleaseGovernanceReason::AuditComplete => "audit_complete",
    }
}
