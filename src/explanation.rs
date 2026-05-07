use crate::admission::{
    admit_evidence, AdmissionEnvelope, AdmissionOutcome, AdmissionRecord, AdmissionRejectionReason,
};
use crate::evidence::{
    CompatibilityClassification, CompatibilityGateSummary, ComputeEvidenceKind,
    ComputeEvidenceSummary,
};
use crate::governance::{
    disposition_rank, govern_admission, trust_rank, Disposition, GovernanceReason,
    GovernanceRecord, TrustClass,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplanationSeverity {
    Info,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentKind {
    IntakeRejected,
    EvidenceDegraded,
    PluginIncompatible,
    BackendInadmissible,
    CacheNotReusable,
    BoundaryDrift,
    ReleaseReadinessBlocked,
    BackendRuntimeNeedsParity,
    HumanReviewRequired,
    EvidenceReady,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplanationFragment {
    pub incident_kind: IncidentKind,
    pub severity: ExplanationSeverity,
    pub evidence_key: String,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfacingAudience {
    Operator,
    DownstreamSystem,
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfacingAction {
    ShowOrdinarily,
    Promote,
    Fallback,
    Degrade,
    Escalate,
    HoldForReview,
    Suppress,
    Refuse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacingDirective {
    pub audience: SurfacingAudience,
    pub action: SurfacingAction,
    pub headline: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramHint {
    pub wires: Vec<String>,
    pub boxes: Vec<String>,
    pub traces: Vec<String>,
    pub contractions: Vec<String>,
    pub splits: Vec<String>,
    pub evidence_flow: Vec<DiagramEvidenceFlow>,
    pub latent_evidence_keys: Vec<String>,
    pub interpretation_steps: Vec<DiagramInterpretationStep>,
    pub posture_flow: Vec<DiagramPostureFlow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramEvidenceFlow {
    pub evidence_key: String,
    pub evidence_kind: ComputeEvidenceKind,
    pub admission_outcome: AdmissionOutcome,
    pub rejection_reasons: Vec<AdmissionRejectionReason>,
    pub disposition: Option<Disposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramInterpretationStepKind {
    Admission,
    TrustAssignment,
    DispositionAssignment,
    SurfacingAssembly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramOutcomeKind {
    Admitted,
    Rejected,
    Ready,
    ReviewRequired,
    Degraded,
    FallbackOnly,
    Refused,
    Promoted,
    Fallback,
    Suppressed,
    Escalated,
    HeldForReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramInterpretationStep {
    pub step: DiagramInterpretationStepKind,
    pub evidence_key: String,
    pub outcome: DiagramOutcomeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramPostureChannel {
    BackendAdmissibility,
    BackendRuntime,
    PluginCompatibility,
    CacheRecompute,
    CompatibilityGate,
    ReleaseReadiness,
    GenericArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramPostureFlow {
    pub channel: DiagramPostureChannel,
    pub evidence_key: String,
    pub outcome: DiagramOutcomeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlTrace {
    pub trace_key: String,
    pub workflow_context: String,
    pub envelopes: Vec<AdmissionEnvelope>,
}

impl ControlTrace {
    pub fn new(
        trace_key: impl Into<String>,
        workflow_context: impl Into<String>,
        envelopes: Vec<AdmissionEnvelope>,
    ) -> Result<Self, &'static str> {
        let trace_key = trace_key.into();
        let workflow_context = workflow_context.into();
        if trace_key.trim().is_empty() {
            return Err("control_trace trace_key must not be empty");
        }
        if workflow_context.trim().is_empty() {
            return Err("control_trace workflow_context must not be empty");
        }
        if envelopes.is_empty() {
            return Err("control_trace envelopes must not be empty");
        }
        Ok(Self {
            trace_key,
            workflow_context,
            envelopes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplanationBundle {
    pub bundle_key: String,
    pub trace_key: String,
    pub workflow_context: String,
    pub admission_records: Vec<AdmissionRecord>,
    pub governance_records: Vec<GovernanceRecord>,
    pub trust_class: TrustClass,
    pub disposition: Disposition,
    pub fragments: Vec<ExplanationFragment>,
    pub surfacing: Vec<SurfacingDirective>,
    pub audit_summary: String,
    pub diagram_hint: DiagramHint,
}

pub fn assemble_explanation_bundle(
    trace: &ControlTrace,
) -> Result<ExplanationBundle, &'static str> {
    if trace
        .envelopes
        .iter()
        .any(|envelope| envelope.provenance.workflow_context != trace.workflow_context)
    {
        return Err("control_trace envelope workflow_context must match trace workflow_context");
    }

    let mut admission_records = Vec::new();
    let mut governance_records = Vec::new();
    let mut fragments = Vec::new();
    let mut trust_class = TrustClass::Ready;
    let mut disposition = Disposition::Promote;

    for envelope in &trace.envelopes {
        let admission = admit_evidence(envelope);
        let governance = govern_admission(envelope, &admission);

        if trust_rank(governance.trust_class) > trust_rank(trust_class) {
            trust_class = governance.trust_class;
        }
        if disposition_rank(governance.disposition) > disposition_rank(disposition) {
            disposition = governance.disposition;
        }

        fragments.extend(fragments_for(envelope, &admission, &governance));
        admission_records.push(admission);
        governance_records.push(governance);
    }

    let surfacing = surfacing_for(trace, trust_class, disposition);
    let audit_summary = format!(
        "trace={} workflow={} evidence_count={} trust={} disposition={}",
        trace.trace_key,
        trace.workflow_context,
        trace.envelopes.len(),
        trust_class_id(trust_class),
        disposition_id(disposition)
    );
    let diagram_hint = diagram_hint_for(trace, &admission_records, &governance_records);

    Ok(ExplanationBundle {
        bundle_key: format!("explanation/{}", trace.trace_key),
        trace_key: trace.trace_key.clone(),
        workflow_context: trace.workflow_context.clone(),
        admission_records,
        governance_records,
        trust_class,
        disposition,
        fragments,
        surfacing,
        audit_summary,
        diagram_hint,
    })
}

fn fragments_for(
    envelope: &AdmissionEnvelope,
    admission: &AdmissionRecord,
    governance: &GovernanceRecord,
) -> Vec<ExplanationFragment> {
    let mut fragments = Vec::new();
    if admission.outcome == AdmissionOutcome::Rejected {
        fragments.push(ExplanationFragment {
            incident_kind: IncidentKind::IntakeRejected,
            severity: ExplanationSeverity::Blocking,
            evidence_key: envelope.evidence_key.clone(),
            summary: format!(
                "evidence rejected at admission for {}",
                join_admission_rejection_reason_ids(&admission.rejection_reasons)
            ),
        });
        return fragments;
    }

    for reason in &governance.reasons {
        let (incident_kind, severity, summary) = fragment_text(envelope, *reason);
        fragments.push(ExplanationFragment {
            incident_kind,
            severity,
            evidence_key: envelope.evidence_key.clone(),
            summary,
        });
    }
    fragments
}

fn fragment_text(
    envelope: &AdmissionEnvelope,
    reason: GovernanceReason,
) -> (IncidentKind, ExplanationSeverity, String) {
    match reason {
        GovernanceReason::AdmissionRejected => (
            IncidentKind::IntakeRejected,
            ExplanationSeverity::Blocking,
            "evidence did not pass Control intake".to_string(),
        ),
        GovernanceReason::PluginCompatible => (
            IncidentKind::EvidenceReady,
            ExplanationSeverity::Info,
            "plugin compatibility evidence is compatible for the requested Control boundary"
                .to_string(),
        ),
        GovernanceReason::PluginIncompatible => (
            IncidentKind::PluginIncompatible,
            ExplanationSeverity::Blocking,
            "plugin compatibility evidence is valid but incompatible for workflow consequence"
                .to_string(),
        ),
        GovernanceReason::BackendInadmissible => (
            IncidentKind::BackendInadmissible,
            ExplanationSeverity::Blocking,
            "backend admissibility evidence is valid but inadmissible for this target".to_string(),
        ),
        GovernanceReason::BackendAdmissible => (
            IncidentKind::EvidenceReady,
            ExplanationSeverity::Info,
            "backend admissibility evidence is admissible for this target".to_string(),
        ),
        GovernanceReason::CacheFresh => (
            IncidentKind::EvidenceReady,
            ExplanationSeverity::Info,
            "cache posture is fresh and reuse-admissible".to_string(),
        ),
        GovernanceReason::CacheStale => (
            IncidentKind::CacheNotReusable,
            ExplanationSeverity::Warning,
            "cache posture is not fresh; workflow consequence must account for recompute or degradation"
                .to_string(),
        ),
        GovernanceReason::CacheRetired => (
            IncidentKind::CacheNotReusable,
            ExplanationSeverity::Blocking,
            "cache posture marks the artifact retired".to_string(),
        ),
        GovernanceReason::CompatibilityGateClean => (
            IncidentKind::EvidenceReady,
            ExplanationSeverity::Info,
            "compatibility gate reports no boundary drift".to_string(),
        ),
        GovernanceReason::CompatibilityGateBlocking => {
            let detail = compatibility_gate_summary(&envelope.summary, true);
            (IncidentKind::BoundaryDrift, ExplanationSeverity::Blocking, detail)
        }
        GovernanceReason::CompatibilityGateNeedsReview => (
            IncidentKind::BoundaryDrift,
            ExplanationSeverity::Warning,
            compatibility_gate_summary(&envelope.summary, false),
        ),
        GovernanceReason::ReadinessNeedsReview => (
            IncidentKind::HumanReviewRequired,
            ExplanationSeverity::Warning,
            release_readiness_summary(&envelope.summary, "needs review before workflow consequence"),
        ),
        GovernanceReason::ReadinessBlocked => (
            IncidentKind::ReleaseReadinessBlocked,
            ExplanationSeverity::Blocking,
            release_readiness_summary(&envelope.summary, "is blocked"),
        ),
        GovernanceReason::BackendRuntimeReady => (
            IncidentKind::EvidenceReady,
            ExplanationSeverity::Info,
            "backend runtime posture preserves CPU/reference oracle and backend-independent canonicalization"
                .to_string(),
        ),
        GovernanceReason::BackendRuntimeNeedsParity => (
            IncidentKind::BackendRuntimeNeedsParity,
            ExplanationSeverity::Warning,
            "backend runtime posture needs parity, layout, precision, or canonicalization review"
                .to_string(),
        ),
        GovernanceReason::GenericArtifactNeedsReview => (
            IncidentKind::HumanReviewRequired,
            ExplanationSeverity::Warning,
            "generic compute artifact is admitted but requires Control review before workflow consequence"
                .to_string(),
        ),
        GovernanceReason::EvidenceNonComparable => (
            IncidentKind::HumanReviewRequired,
            ExplanationSeverity::Warning,
            "compute evidence is non-comparable and requires explicit Control review".to_string(),
        ),
        GovernanceReason::EvidenceReady => (
            IncidentKind::EvidenceReady,
            ExplanationSeverity::Info,
            "compute evidence is admitted and ready for ordinary workflow consideration".to_string(),
        ),
        GovernanceReason::EvidenceDegraded => (
            IncidentKind::EvidenceDegraded,
            ExplanationSeverity::Warning,
            "compute evidence is admitted but degraded for Control purposes".to_string(),
        ),
        GovernanceReason::ComputeRefused => (
            IncidentKind::IntakeRejected,
            ExplanationSeverity::Blocking,
            "compute evidence reports refused semantics and is refused for workflow consequence"
                .to_string(),
        ),
    }
}

fn surfacing_for(
    trace: &ControlTrace,
    trust_class: TrustClass,
    disposition: Disposition,
) -> Vec<SurfacingDirective> {
    let action = surfacing_action_for(disposition);
    vec![
        SurfacingDirective {
            audience: SurfacingAudience::Operator,
            action,
            headline: format!("Control disposition: {}", disposition_id(disposition)),
            body: format!(
                "Trace {} in {} has trust {}; workflow consequence requires explicit Control handling.",
                trace.trace_key, trace.workflow_context, trust_class_id(trust_class)
            ),
        },
        SurfacingDirective {
            audience: SurfacingAudience::DownstreamSystem,
            action,
            headline: disposition_id(disposition).to_string(),
            body: downstream_body_for(disposition).to_string(),
        },
        SurfacingDirective {
            audience: SurfacingAudience::Audit,
            action: SurfacingAction::ShowOrdinarily,
            headline: "Workflow-auditable explanation bundle".to_string(),
            body: format!("{} evidence envelopes assembled", trace.envelopes.len()),
        },
    ]
}

fn surfacing_action_for(disposition: Disposition) -> SurfacingAction {
    match disposition {
        Disposition::Promote => SurfacingAction::Promote,
        Disposition::Fallback => SurfacingAction::Fallback,
        Disposition::Refuse => SurfacingAction::Refuse,
        Disposition::Suppress => SurfacingAction::Suppress,
        Disposition::Degrade => SurfacingAction::Degrade,
        Disposition::Escalate => SurfacingAction::Escalate,
        Disposition::HoldForReview => SurfacingAction::HoldForReview,
    }
}

fn join_admission_rejection_reason_ids(reasons: &[AdmissionRejectionReason]) -> String {
    reasons
        .iter()
        .copied()
        .map(admission_rejection_reason_id)
        .collect::<Vec<_>>()
        .join(",")
}

fn admission_rejection_reason_id(reason: AdmissionRejectionReason) -> &'static str {
    match reason {
        AdmissionRejectionReason::MissingEvidenceKey => "missing_evidence_key",
        AdmissionRejectionReason::EvidenceKindMismatch => "evidence_kind_mismatch",
        AdmissionRejectionReason::MissingArtifactIdentity => "missing_artifact_identity",
        AdmissionRejectionReason::MissingSourceSystem => "missing_source_system",
        AdmissionRejectionReason::MissingWorkflowContext => "missing_workflow_context",
        AdmissionRejectionReason::MissingReplayReference => "missing_replay_reference",
        AdmissionRejectionReason::MissingLineage => "missing_lineage",
        AdmissionRejectionReason::MalformedContentHash => "malformed_content_hash",
        AdmissionRejectionReason::ComputeValidationFailed => "compute_validation_failed",
        AdmissionRejectionReason::StaleDigest => "stale_digest",
        AdmissionRejectionReason::UnknownValidationPosture => "unknown_validation_posture",
        AdmissionRejectionReason::SemanticallyInadmissible => "semantically_inadmissible",
        AdmissionRejectionReason::CacheDependencyMissing => "cache_dependency_missing",
        AdmissionRejectionReason::DependencyContentDrift => "dependency_content_drift",
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

fn compatibility_gate_summary(summary: &ComputeEvidenceSummary, blocking: bool) -> String {
    match summary {
        ComputeEvidenceSummary::ContractCompatibilityGate(summary) => {
            format_compatibility_gate_summary(summary, blocking)
        }
        _ if blocking => "compatibility gate is blocking".to_string(),
        _ => "compatibility gate needs review".to_string(),
    }
}

fn format_compatibility_gate_summary(summary: &CompatibilityGateSummary, blocking: bool) -> String {
    let posture = if blocking { "blocking" } else { "needs_review" };
    format!(
        "compatibility gate {}: classification={} reasons={}",
        posture,
        compatibility_classification_id(summary.classification),
        summary.reasons.join(",")
    )
}

fn release_readiness_summary(summary: &ComputeEvidenceSummary, posture: &str) -> String {
    match summary {
        ComputeEvidenceSummary::ReleaseReadiness(summary) => {
            format!(
                "release readiness {posture} ({})",
                summary.reasons.join(",")
            )
        }
        _ => format!("release readiness {posture}"),
    }
}

fn diagram_hint_for(
    trace: &ControlTrace,
    admission_records: &[AdmissionRecord],
    governance_records: &[GovernanceRecord],
) -> DiagramHint {
    let mut boxes = vec![
        "admission".to_string(),
        "trust_assignment".to_string(),
        "disposition_assignment".to_string(),
        "explanation_assembly".to_string(),
    ];
    if trace.envelopes.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::BackendAdmissibility { .. }
        )
    }) {
        boxes.push("backend_admissibility".to_string());
    }
    if trace.envelopes.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::PluginCompatibility(_)
        )
    }) {
        boxes.push("plugin_compatibility".to_string());
    }
    if trace.envelopes.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::ContractCompatibilityGate(_)
        )
    }) {
        boxes.push("compatibility_verdict".to_string());
    }
    if trace
        .envelopes
        .iter()
        .any(|envelope| matches!(envelope.summary, ComputeEvidenceSummary::CachePolicy(_)))
    {
        boxes.push("cache_recompute_posture".to_string());
    }
    if trace.envelopes.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::ReleaseReadiness(_)
        )
    }) {
        boxes.push("release_readiness".to_string());
    }
    if trace.envelopes.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::BackendRuntimePosture(_)
        )
    }) {
        boxes.push("backend_runtime_posture".to_string());
    }
    if trace.envelopes.iter().any(|envelope| {
        matches!(
            envelope.summary,
            ComputeEvidenceSummary::GenericArtifact { .. }
        )
    }) {
        boxes.push("generic_artifact_review".to_string());
    }

    let evidence_flow = trace
        .envelopes
        .iter()
        .zip(admission_records)
        .zip(governance_records)
        .map(|((envelope, admission), governance)| DiagramEvidenceFlow {
            evidence_key: envelope.evidence_key.clone(),
            evidence_kind: envelope.evidence_kind,
            admission_outcome: admission.outcome,
            rejection_reasons: admission.rejection_reasons.clone(),
            disposition: (admission.outcome == AdmissionOutcome::Admitted)
                .then_some(governance.disposition),
        })
        .collect::<Vec<_>>();

    let latent_evidence_keys = evidence_flow
        .iter()
        .filter(|flow| flow.admission_outcome == AdmissionOutcome::Rejected)
        .map(|flow| flow.evidence_key.clone())
        .collect::<Vec<_>>();

    let mut interpretation_steps = Vec::new();
    let mut posture_flow = Vec::new();
    for ((envelope, admission), governance) in trace
        .envelopes
        .iter()
        .zip(admission_records)
        .zip(governance_records)
    {
        interpretation_steps.push(DiagramInterpretationStep {
            step: DiagramInterpretationStepKind::Admission,
            evidence_key: envelope.evidence_key.clone(),
            outcome: admission_outcome_kind(admission.outcome),
        });
        if admission.outcome == AdmissionOutcome::Admitted {
            interpretation_steps.push(DiagramInterpretationStep {
                step: DiagramInterpretationStepKind::TrustAssignment,
                evidence_key: envelope.evidence_key.clone(),
                outcome: trust_outcome_kind(governance.trust_class),
            });
            interpretation_steps.push(DiagramInterpretationStep {
                step: DiagramInterpretationStepKind::DispositionAssignment,
                evidence_key: envelope.evidence_key.clone(),
                outcome: disposition_outcome_kind(governance.disposition),
            });
            posture_flow.push(DiagramPostureFlow {
                channel: posture_channel_for(&envelope.summary),
                evidence_key: envelope.evidence_key.clone(),
                outcome: disposition_outcome_kind(governance.disposition),
            });
        }
    }
    interpretation_steps.push(DiagramInterpretationStep {
        step: DiagramInterpretationStepKind::SurfacingAssembly,
        evidence_key: trace.trace_key.clone(),
        outcome: disposition_outcome_kind(
            governance_records
                .iter()
                .map(|record| record.disposition)
                .max_by_key(|disposition| disposition_rank(*disposition))
                .unwrap_or(Disposition::HoldForReview),
        ),
    });

    DiagramHint {
        wires: vec![
            "compute_evidence".to_string(),
            "admission_state".to_string(),
            "trust_class".to_string(),
            "workflow_context".to_string(),
            "control_disposition".to_string(),
        ],
        boxes,
        traces: vec![
            "unadmitted_evidence_is_latent".to_string(),
            "rejected_evidence_does_not_become_workflow_truth".to_string(),
        ],
        contractions: vec![
            "admitted_evidence_to_governance".to_string(),
            "evidence_chain_to_explanation_bundle".to_string(),
        ],
        splits: vec![
            "compute_truth_vs_workflow_truth".to_string(),
            "admission_vs_approval".to_string(),
        ],
        evidence_flow,
        latent_evidence_keys,
        interpretation_steps,
        posture_flow,
    }
}

fn downstream_body_for(disposition: Disposition) -> &'static str {
    match disposition {
        Disposition::Promote => {
            "Control posture=promote; downstream systems may treat the evidence as eligible for explicit workflow consequence."
        }
        Disposition::Fallback => {
            "Control posture=fallback; downstream systems may use the evidence only as fallback context."
        }
        Disposition::Refuse => {
            "Control posture=refuse; downstream systems must not use the evidence for workflow consequence."
        }
        Disposition::Suppress => {
            "Control posture=suppress; downstream systems must withhold ordinary surfacing while preserving auditability."
        }
        Disposition::Degrade => {
            "Control posture=degrade; downstream systems must carry reduced operational standing."
        }
        Disposition::Escalate => {
            "Control posture=escalate; downstream systems must route for higher scrutiny before consequence."
        }
        Disposition::HoldForReview => {
            "Control posture=hold_for_review; downstream systems must pause consequence pending explicit review."
        }
    }
}

fn trust_class_id(trust_class: TrustClass) -> &'static str {
    match trust_class {
        TrustClass::Ready => "ready",
        TrustClass::ReviewRequired => "review_required",
        TrustClass::Degraded => "degraded",
        TrustClass::FallbackOnly => "fallback_only",
        TrustClass::Refused => "refused",
    }
}

fn disposition_id(disposition: Disposition) -> &'static str {
    match disposition {
        Disposition::Promote => "promote",
        Disposition::Fallback => "fallback",
        Disposition::Refuse => "refuse",
        Disposition::Suppress => "suppress",
        Disposition::Degrade => "degrade",
        Disposition::Escalate => "escalate",
        Disposition::HoldForReview => "hold_for_review",
    }
}

fn admission_outcome_kind(outcome: AdmissionOutcome) -> DiagramOutcomeKind {
    match outcome {
        AdmissionOutcome::Admitted => DiagramOutcomeKind::Admitted,
        AdmissionOutcome::Rejected => DiagramOutcomeKind::Rejected,
    }
}

fn trust_outcome_kind(trust_class: TrustClass) -> DiagramOutcomeKind {
    match trust_class {
        TrustClass::Ready => DiagramOutcomeKind::Ready,
        TrustClass::ReviewRequired => DiagramOutcomeKind::ReviewRequired,
        TrustClass::Degraded => DiagramOutcomeKind::Degraded,
        TrustClass::FallbackOnly => DiagramOutcomeKind::FallbackOnly,
        TrustClass::Refused => DiagramOutcomeKind::Refused,
    }
}

fn disposition_outcome_kind(disposition: Disposition) -> DiagramOutcomeKind {
    match disposition {
        Disposition::Promote => DiagramOutcomeKind::Promoted,
        Disposition::Fallback => DiagramOutcomeKind::Fallback,
        Disposition::Refuse => DiagramOutcomeKind::Refused,
        Disposition::Suppress => DiagramOutcomeKind::Suppressed,
        Disposition::Degrade => DiagramOutcomeKind::Degraded,
        Disposition::Escalate => DiagramOutcomeKind::Escalated,
        Disposition::HoldForReview => DiagramOutcomeKind::HeldForReview,
    }
}

fn posture_channel_for(summary: &ComputeEvidenceSummary) -> DiagramPostureChannel {
    match summary {
        ComputeEvidenceSummary::GenericArtifact { .. } => DiagramPostureChannel::GenericArtifact,
        ComputeEvidenceSummary::PluginCompatibility(_) => {
            DiagramPostureChannel::PluginCompatibility
        }
        ComputeEvidenceSummary::CachePolicy(_) => DiagramPostureChannel::CacheRecompute,
        ComputeEvidenceSummary::ContractCompatibilityGate(_) => {
            DiagramPostureChannel::CompatibilityGate
        }
        ComputeEvidenceSummary::ReleaseReadiness(_) => DiagramPostureChannel::ReleaseReadiness,
        ComputeEvidenceSummary::BackendRuntimePosture(_) => DiagramPostureChannel::BackendRuntime,
        ComputeEvidenceSummary::BackendAdmissibility { .. } => {
            DiagramPostureChannel::BackendAdmissibility
        }
    }
}
