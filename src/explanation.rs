use crate::admission::{admit_evidence, AdmissionEnvelope, AdmissionOutcome, AdmissionRecord};
use crate::evidence::ComputeEvidenceSummary;
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
        "trace={} workflow={} evidence_count={} trust={:?} disposition={:?}",
        trace.trace_key,
        trace.workflow_context,
        trace.envelopes.len(),
        trust_class,
        disposition
    );

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
        diagram_hint: diagram_hint_for(trace),
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
                "evidence rejected at admission for {:?}",
                admission.rejection_reasons
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
            IncidentKind::BackendRuntimeNeedsParity,
            ExplanationSeverity::Blocking,
            "backend admissibility evidence is valid but inadmissible for this target".to_string(),
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
            let detail = match &envelope.summary {
                ComputeEvidenceSummary::ContractCompatibilityGate(summary) => {
                    format!(
                        "compatibility gate is {:?} with reasons {:?}",
                        summary.classification, summary.reasons
                    )
                }
                _ => "compatibility gate is blocking".to_string(),
            };
            (IncidentKind::BoundaryDrift, ExplanationSeverity::Blocking, detail)
        }
        GovernanceReason::ReadinessNeedsReview => (
            IncidentKind::HumanReviewRequired,
            ExplanationSeverity::Warning,
            "release or boundary readiness needs review before workflow consequence".to_string(),
        ),
        GovernanceReason::ReadinessBlocked => (
            IncidentKind::ReleaseReadinessBlocked,
            ExplanationSeverity::Blocking,
            "release or boundary readiness is blocked".to_string(),
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
            headline: format!("Control disposition: {:?}", disposition),
            body: format!(
                "Trace {} in {} has trust {:?}; workflow consequence requires explicit Control handling.",
                trace.trace_key, trace.workflow_context, trust_class
            ),
        },
        SurfacingDirective {
            audience: SurfacingAudience::DownstreamSystem,
            action,
            headline: format!("{:?}", disposition),
            body: "consume this as Control-owned workflow posture, not compute truth".to_string(),
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

fn diagram_hint_for(trace: &ControlTrace) -> DiagramHint {
    let mut boxes = vec!["admission".to_string(), "governance".to_string()];
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

    DiagramHint {
        wires: vec![
            "compute_evidence".to_string(),
            "workflow_context".to_string(),
            "control_disposition".to_string(),
        ],
        boxes,
        traces: vec!["unadmitted_evidence_is_not_workflow_truth".to_string()],
        contractions: vec!["evidence_chain_to_explanation_bundle".to_string()],
        splits: vec!["compute_truth_vs_workflow_truth".to_string()],
    }
}
