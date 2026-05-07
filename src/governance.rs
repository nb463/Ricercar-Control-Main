use crate::admission::{AdmissionEnvelope, AdmissionOutcome, AdmissionRecord};
use crate::evidence::{
    BackendAdmissibility, BackendCanonicalizationPosture, BackendMemoryLayoutPosture,
    CacheLifecycleState, CacheReuseAdmissibility, CompatibilityClassification,
    ComputeEvidenceSummary, ComputeSemanticStatus, EvidenceReadiness, PluginCompatibility,
    PrecisionPosture,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustClass {
    Ready,
    ReviewRequired,
    Degraded,
    FallbackOnly,
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    Promote,
    Fallback,
    Refuse,
    Suppress,
    Degrade,
    Escalate,
    HoldForReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceReason {
    AdmissionRejected,
    EvidenceReady,
    EvidenceDegraded,
    ComputeRefused,
    PluginCompatible,
    PluginIncompatible,
    BackendInadmissible,
    CacheFresh,
    CacheStale,
    CacheRetired,
    CompatibilityGateClean,
    CompatibilityGateBlocking,
    ReadinessNeedsReview,
    ReadinessBlocked,
    BackendRuntimeReady,
    BackendRuntimeNeedsParity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceRecord {
    pub evidence_key: String,
    pub trust_class: TrustClass,
    pub disposition: Disposition,
    pub reasons: Vec<GovernanceReason>,
}

pub fn govern_admission(
    envelope: &AdmissionEnvelope,
    admission: &AdmissionRecord,
) -> GovernanceRecord {
    if admission.outcome == AdmissionOutcome::Rejected {
        return GovernanceRecord {
            evidence_key: envelope.evidence_key.clone(),
            trust_class: TrustClass::Refused,
            disposition: Disposition::Refuse,
            reasons: vec![GovernanceReason::AdmissionRejected],
        };
    }

    let mut record = match &envelope.summary {
        ComputeEvidenceSummary::PluginCompatibility(summary) => {
            if summary.compatibility == PluginCompatibility::Compatible {
                GovernanceRecord {
                    evidence_key: envelope.evidence_key.clone(),
                    trust_class: TrustClass::Ready,
                    disposition: Disposition::Promote,
                    reasons: vec![GovernanceReason::PluginCompatible],
                }
            } else {
                GovernanceRecord {
                    evidence_key: envelope.evidence_key.clone(),
                    trust_class: TrustClass::Refused,
                    disposition: Disposition::Refuse,
                    reasons: vec![GovernanceReason::PluginIncompatible],
                }
            }
        }
        ComputeEvidenceSummary::BackendAdmissibility { admissibility, .. } => {
            if *admissibility == BackendAdmissibility::Admissible {
                ready_record(envelope, GovernanceReason::BackendRuntimeReady)
            } else {
                GovernanceRecord {
                    evidence_key: envelope.evidence_key.clone(),
                    trust_class: TrustClass::Refused,
                    disposition: Disposition::Refuse,
                    reasons: vec![GovernanceReason::BackendInadmissible],
                }
            }
        }
        ComputeEvidenceSummary::CachePolicy(summary) => match summary.lifecycle_state {
            CacheLifecycleState::Fresh
                if summary.reuse_admissibility == CacheReuseAdmissibility::ReuseAdmissible =>
            {
                ready_record(envelope, GovernanceReason::CacheFresh)
            }
            CacheLifecycleState::Retired => GovernanceRecord {
                evidence_key: envelope.evidence_key.clone(),
                trust_class: TrustClass::Refused,
                disposition: Disposition::Refuse,
                reasons: vec![GovernanceReason::CacheRetired],
            },
            _ => GovernanceRecord {
                evidence_key: envelope.evidence_key.clone(),
                trust_class: TrustClass::Degraded,
                disposition: Disposition::Degrade,
                reasons: vec![GovernanceReason::CacheStale],
            },
        },
        ComputeEvidenceSummary::ContractCompatibilityGate(summary) => {
            if summary.gate_blocking
                || summary.classification != CompatibilityClassification::InternalOnly
            {
                GovernanceRecord {
                    evidence_key: envelope.evidence_key.clone(),
                    trust_class: TrustClass::ReviewRequired,
                    disposition: Disposition::Escalate,
                    reasons: vec![GovernanceReason::CompatibilityGateBlocking],
                }
            } else {
                ready_record(envelope, GovernanceReason::CompatibilityGateClean)
            }
        }
        ComputeEvidenceSummary::ReleaseReadiness(summary) => match summary.readiness {
            EvidenceReadiness::Ready => ready_record(envelope, GovernanceReason::EvidenceReady),
            EvidenceReadiness::NeedsReview => GovernanceRecord {
                evidence_key: envelope.evidence_key.clone(),
                trust_class: TrustClass::ReviewRequired,
                disposition: Disposition::HoldForReview,
                reasons: vec![GovernanceReason::ReadinessNeedsReview],
            },
            EvidenceReadiness::Blocked => GovernanceRecord {
                evidence_key: envelope.evidence_key.clone(),
                trust_class: TrustClass::ReviewRequired,
                disposition: Disposition::Escalate,
                reasons: vec![GovernanceReason::ReadinessBlocked],
            },
            EvidenceReadiness::NotApplicable => {
                ready_record(envelope, GovernanceReason::EvidenceReady)
            }
        },
        ComputeEvidenceSummary::BackendRuntimePosture(summary) => {
            if summary.layout_posture == BackendMemoryLayoutPosture::VersionMismatch
                || summary.precision_posture == PrecisionPosture::Mismatch
                || summary.canonicalization_posture
                    != BackendCanonicalizationPosture::BackendIndependent
                || !summary.parity_oracle.contains("cpu_reference")
            {
                GovernanceRecord {
                    evidence_key: envelope.evidence_key.clone(),
                    trust_class: TrustClass::ReviewRequired,
                    disposition: Disposition::HoldForReview,
                    reasons: vec![GovernanceReason::BackendRuntimeNeedsParity],
                }
            } else {
                ready_record(envelope, GovernanceReason::BackendRuntimeReady)
            }
        }
        ComputeEvidenceSummary::GenericArtifact { .. } => {
            ready_record(envelope, GovernanceReason::EvidenceReady)
        }
    };

    match envelope.semantic_status {
        ComputeSemanticStatus::Lawful => {}
        ComputeSemanticStatus::Degraded => {
            soften_record(
                &mut record,
                TrustClass::Degraded,
                Disposition::Degrade,
                GovernanceReason::EvidenceDegraded,
            );
        }
        ComputeSemanticStatus::Refused => {
            soften_record(
                &mut record,
                TrustClass::Refused,
                Disposition::Refuse,
                GovernanceReason::ComputeRefused,
            );
        }
        ComputeSemanticStatus::NonComparable | ComputeSemanticStatus::Unknown => {}
    }

    record
}

fn ready_record(envelope: &AdmissionEnvelope, reason: GovernanceReason) -> GovernanceRecord {
    GovernanceRecord {
        evidence_key: envelope.evidence_key.clone(),
        trust_class: TrustClass::Ready,
        disposition: Disposition::Promote,
        reasons: vec![reason],
    }
}

fn soften_record(
    record: &mut GovernanceRecord,
    trust_class: TrustClass,
    disposition: Disposition,
    reason: GovernanceReason,
) {
    if trust_rank(trust_class) > trust_rank(record.trust_class) {
        record.trust_class = trust_class;
    }
    if disposition_rank(disposition) > disposition_rank(record.disposition) {
        record.disposition = disposition;
    }
    if !record.reasons.contains(&reason) {
        record.reasons.push(reason);
    }
}

pub(crate) fn trust_rank(trust: TrustClass) -> u8 {
    match trust {
        TrustClass::Ready => 0,
        TrustClass::FallbackOnly => 1,
        TrustClass::Degraded => 2,
        TrustClass::ReviewRequired => 3,
        TrustClass::Refused => 4,
    }
}

pub(crate) fn disposition_rank(disposition: Disposition) -> u8 {
    match disposition {
        Disposition::Promote => 0,
        Disposition::Fallback => 1,
        Disposition::Degrade => 2,
        Disposition::Suppress => 3,
        Disposition::HoldForReview => 4,
        Disposition::Escalate => 5,
        Disposition::Refuse => 6,
    }
}
