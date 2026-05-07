pub mod admission;
pub mod evidence;
pub mod explanation;
pub mod governance;

pub use admission::{
    admit_evidence, AdmissionEnvelope, AdmissionOutcome, AdmissionRecord, AdmissionRejectionReason,
};
pub use evidence::{
    BackendAdmissibility, BackendCanonicalizationPosture, BackendMemoryLayoutPosture, BackendRole,
    BackendRuntimePostureSummary, CacheBlockedReason, CacheCoherencePosture, CacheLifecycleState,
    CachePolicySummary, CacheReuseAdmissibility, CompatibilityClassification,
    CompatibilityGateSummary, ComputeEvidenceKind, ComputeEvidenceSummary, ComputeSemanticStatus,
    ComputeValidationPosture, EvidenceProvenance, EvidenceReadiness, PluginCompatibility,
    PluginCompatibilityReason, PluginCompatibilitySummary, PrecisionPosture, RecomputeReason,
    ReleaseReadinessSummary,
};
pub use explanation::{
    assemble_explanation_bundle, ControlTrace, DiagramHint, ExplanationBundle, ExplanationFragment,
    ExplanationSeverity, IncidentKind, SurfacingAction, SurfacingAudience, SurfacingDirective,
};
pub use governance::{
    govern_admission, Disposition, GovernanceReason, GovernanceRecord, TrustClass,
};
