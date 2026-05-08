pub mod admission;
pub mod evidence;
pub mod explanation;
pub mod governance;
pub mod orchestration;

pub use admission::{
    admit_evidence, AdmissionEnvelope, AdmissionOutcome, AdmissionRecord, AdmissionRejectionReason,
};
pub use evidence::{
    BackendAdmissibility, BackendCanonicalizationPosture, BackendLayoutCompatibility,
    BackendLayoutVersion, BackendMemoryLayoutPosture, BackendPackingPolicy, BackendParityOracle,
    BackendPrecisionMode, BackendRole, BackendRuntimePostureSummary, BackendRuntimeTrack,
    CacheBlockedReason, CacheCoherencePosture, CacheLifecycleState, CachePolicySummary,
    CacheReuseAdmissibility, CompatibilityClassification, CompatibilityGateSummary,
    ComputeEvidenceKind, ComputeEvidenceSummary, ComputeSemanticStatus, ComputeValidationPosture,
    CudaBackendPromotionSummary, CudaCanonicalizationPosture, CudaParityBudget, CudaParityStatus,
    CudaPromotionPosture, CudaPromotionReason, CudaWorkloadEligibility,
    CudaWorkloadEligibilityReason, EvidenceProvenance, EvidenceReadiness,
    HostDeviceTransferSemantics, PluginCompatibility, PluginCompatibilityReason,
    PluginCompatibilitySummary, PrecisionPosture, RecomputeReason, ReleaseReadinessSummary,
};
pub use explanation::{
    assemble_explanation_bundle, ControlTrace, DiagramEvidenceFlow, DiagramHint,
    DiagramInterpretationStep, DiagramInterpretationStepKind, DiagramOutcomeKind,
    DiagramPostureChannel, DiagramPostureFlow, ExplanationBundle, ExplanationFragment,
    ExplanationSeverity, IncidentKind, SurfacingAction, SurfacingAudience, SurfacingDirective,
};
pub use governance::{
    govern_admission, Disposition, GovernanceReason, GovernanceRecord, TrustClass,
};
pub use orchestration::{
    execution_command_kind_id, route_work_item, routing_reason_id, ExecutionCommand,
    ExecutionCommandKind, OrchestrationAuditRecord, OrchestrationState, QueuePriority,
    QueueableWorkItem, RoutingDecision, RoutingExplanationPayload, RoutingIntentKind,
    RoutingReason,
};
