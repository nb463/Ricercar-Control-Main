#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcePlane {
    Compute,
    Control,
    Probe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeNodeRole {
    ComputeTruth,
    ControlTruth,
    ProbeOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeEdgeKind {
    DerivedFrom,
    Explains,
    Blocks,
    Changes,
    DrillsDownTo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeWalkthroughKind {
    ShowMeWhy,
    ShowMeWhatChanged,
    ShowMeWhatBlockedPromotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorProbeQuestion {
    WhatHappened,
    WhyHeld,
    WhyDegraded,
    WhyFallbackOnly,
    WhyRollbackRequired,
    WhatChanged,
    WhatBlockedPromotion,
    EvidenceForConsequence,
    OwnershipSplit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorProbeView {
    pub scenario_key: String,
    pub question: OperatorProbeQuestion,
    pub answer_summary: String,
    pub compute_truth_refs: Vec<String>,
    pub control_consequence_refs: Vec<String>,
    pub probe_envelope_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeDeltaCause {
    NoChange,
    ComputeTruthChanged,
    ControlConsequenceChanged,
    MixedOwnershipChanged,
    ProbeOnlyChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeDeltaSummary {
    pub cause: ProbeDeltaCause,
    pub changed_source_ids: Vec<String>,
    pub compute_truth_refs: Vec<String>,
    pub control_consequence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeNode {
    pub node_id: String,
    pub role: ProbeNodeRole,
    pub source: crate::SourceRef,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
}

impl ProbeNode {
    pub fn new(
        node_id: impl Into<String>,
        role: ProbeNodeRole,
        source: crate::SourceRef,
        title: impl Into<String>,
        summary: impl Into<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            role,
            source,
            title: title.into(),
            summary: summary.into(),
            tags,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeEdge {
    pub from_node: String,
    pub to_node: String,
    pub kind: ProbeEdgeKind,
    pub label: String,
}

impl ProbeEdge {
    pub fn new(
        from_node: impl Into<String>,
        to_node: impl Into<String>,
        kind: ProbeEdgeKind,
        label: impl Into<String>,
    ) -> Self {
        Self {
            from_node: from_node.into(),
            to_node: to_node.into(),
            kind,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeTrace {
    pub trace_id: String,
    pub title: String,
    pub node_ids: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeChange {
    pub source_id: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeExplanation {
    pub walkthrough: ProbeWalkthroughKind,
    pub summary: String,
    pub changed: Vec<ProbeChange>,
    pub blocking_reason_ids: Vec<String>,
}

pub fn operator_probe_question_id(question: OperatorProbeQuestion) -> &'static str {
    match question {
        OperatorProbeQuestion::WhatHappened => "what_happened",
        OperatorProbeQuestion::WhyHeld => "why_held",
        OperatorProbeQuestion::WhyDegraded => "why_degraded",
        OperatorProbeQuestion::WhyFallbackOnly => "why_fallback_only",
        OperatorProbeQuestion::WhyRollbackRequired => "why_rollback_required",
        OperatorProbeQuestion::WhatChanged => "what_changed",
        OperatorProbeQuestion::WhatBlockedPromotion => "what_blocked_promotion",
        OperatorProbeQuestion::EvidenceForConsequence => "evidence_for_consequence",
        OperatorProbeQuestion::OwnershipSplit => "ownership_split",
    }
}

pub fn probe_walkthrough_kind_id(kind: ProbeWalkthroughKind) -> &'static str {
    match kind {
        ProbeWalkthroughKind::ShowMeWhy => "show_me_why",
        ProbeWalkthroughKind::ShowMeWhatChanged => "show_me_what_changed",
        ProbeWalkthroughKind::ShowMeWhatBlockedPromotion => "show_me_what_blocked_promotion",
    }
}

pub fn probe_delta_cause_id(cause: ProbeDeltaCause) -> &'static str {
    match cause {
        ProbeDeltaCause::NoChange => "no_change",
        ProbeDeltaCause::ComputeTruthChanged => "compute_truth_changed",
        ProbeDeltaCause::ControlConsequenceChanged => "control_consequence_changed",
        ProbeDeltaCause::MixedOwnershipChanged => "mixed_ownership_changed",
        ProbeDeltaCause::ProbeOnlyChanged => "probe_only_changed",
    }
}
