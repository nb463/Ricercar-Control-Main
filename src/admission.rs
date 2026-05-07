use crate::evidence::{
    is_strict_content_hash, CacheBlockedReason, CacheCoherencePosture, CacheLifecycleState,
    ComputeEvidenceKind, ComputeEvidenceSummary, ComputeSemanticStatus, ComputeValidationPosture,
    EvidenceProvenance,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionEnvelope {
    pub evidence_key: String,
    pub evidence_kind: ComputeEvidenceKind,
    pub provenance: EvidenceProvenance,
    pub validation_posture: ComputeValidationPosture,
    pub semantic_status: ComputeSemanticStatus,
    pub summary: ComputeEvidenceSummary,
}

impl AdmissionEnvelope {
    pub fn new(
        evidence_key: impl Into<String>,
        evidence_kind: ComputeEvidenceKind,
        provenance: EvidenceProvenance,
        validation_posture: ComputeValidationPosture,
        semantic_status: ComputeSemanticStatus,
        summary: ComputeEvidenceSummary,
    ) -> Self {
        Self {
            evidence_key: evidence_key.into(),
            evidence_kind,
            provenance,
            validation_posture,
            semantic_status,
            summary,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionOutcome {
    Admitted,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionRejectionReason {
    MissingEvidenceKey,
    EvidenceKindMismatch,
    MissingArtifactIdentity,
    MissingProvenance,
    MissingReplayReference,
    MissingLineage,
    MalformedContentHash,
    ComputeValidationFailed,
    StaleDigest,
    UnknownValidationPosture,
    SemanticallyInadmissible,
    CacheDependencyMissing,
    DependencyContentDrift,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionRecord {
    pub evidence_key: String,
    pub outcome: AdmissionOutcome,
    pub rejection_reasons: Vec<AdmissionRejectionReason>,
}

pub fn admit_evidence(envelope: &AdmissionEnvelope) -> AdmissionRecord {
    let mut reasons = Vec::new();

    if envelope.evidence_key.trim().is_empty() {
        reasons.push(AdmissionRejectionReason::MissingEvidenceKey);
    }
    if envelope.evidence_kind != envelope.summary.evidence_kind() {
        reasons.push(AdmissionRejectionReason::EvidenceKindMismatch);
    }
    if envelope.provenance.artifact_key.trim().is_empty() {
        reasons.push(AdmissionRejectionReason::MissingArtifactIdentity);
    }
    if envelope.provenance.source_system.trim().is_empty()
        || envelope.provenance.workflow_context.trim().is_empty()
    {
        reasons.push(AdmissionRejectionReason::MissingProvenance);
    }
    if envelope.provenance.replay_ref.trim().is_empty() {
        reasons.push(AdmissionRejectionReason::MissingReplayReference);
    }
    if envelope.provenance.lineage.is_empty()
        || envelope
            .provenance
            .lineage
            .iter()
            .any(|lineage| lineage.trim().is_empty())
    {
        reasons.push(AdmissionRejectionReason::MissingLineage);
    }
    if !is_strict_content_hash(&envelope.provenance.content_hash) {
        reasons.push(AdmissionRejectionReason::MalformedContentHash);
    }

    match envelope.validation_posture {
        ComputeValidationPosture::Validated => {}
        ComputeValidationPosture::Invalid | ComputeValidationPosture::Malformed => {
            reasons.push(AdmissionRejectionReason::ComputeValidationFailed);
        }
        ComputeValidationPosture::StaleDigest => {
            reasons.push(AdmissionRejectionReason::StaleDigest);
        }
        ComputeValidationPosture::Unknown => {
            reasons.push(AdmissionRejectionReason::UnknownValidationPosture);
        }
    }

    if matches!(
        envelope.semantic_status,
        ComputeSemanticStatus::Unknown | ComputeSemanticStatus::NonComparable
    ) {
        reasons.push(AdmissionRejectionReason::SemanticallyInadmissible);
    }

    if let ComputeEvidenceSummary::CachePolicy(summary) = &envelope.summary {
        if matches!(
            summary.lifecycle_state,
            CacheLifecycleState::BlockedDependencyMissing
        ) || summary.blocked_reason == Some(CacheBlockedReason::DependencyMissing)
            || summary.coherence_posture == CacheCoherencePosture::DependencyMissing
        {
            reasons.push(AdmissionRejectionReason::CacheDependencyMissing);
        }
        if summary.coherence_posture == CacheCoherencePosture::DependencyContentDrift {
            reasons.push(AdmissionRejectionReason::DependencyContentDrift);
        }
    }

    let outcome = if reasons.is_empty() {
        AdmissionOutcome::Admitted
    } else {
        AdmissionOutcome::Rejected
    };

    AdmissionRecord {
        evidence_key: envelope.evidence_key.clone(),
        outcome,
        rejection_reasons: reasons,
    }
}
