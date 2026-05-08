#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeEvidenceKind {
    ComputeArtifact,
    PluginCompatibility,
    CachePolicy,
    ContractCompatibilityGate,
    ReleaseReadiness,
    BackendRuntimePosture,
    BackendAdmissibility,
    CudaBackendPromotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeValidationPosture {
    Validated,
    Invalid,
    Malformed,
    StaleDigest,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeSemanticStatus {
    Lawful,
    Degraded,
    Refused,
    NonComparable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceProvenance {
    pub source_system: String,
    pub workflow_context: String,
    pub artifact_key: String,
    pub content_hash: String,
    pub replay_ref: String,
    pub lineage: Vec<String>,
}

impl EvidenceProvenance {
    pub fn new(
        source_system: impl Into<String>,
        workflow_context: impl Into<String>,
        artifact_key: impl Into<String>,
        content_hash: impl Into<String>,
        replay_ref: impl Into<String>,
        lineage: Vec<String>,
    ) -> Self {
        Self {
            source_system: source_system.into(),
            workflow_context: workflow_context.into(),
            artifact_key: artifact_key.into(),
            content_hash: content_hash.into(),
            replay_ref: replay_ref.into(),
            lineage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputeEvidenceSummary {
    GenericArtifact {
        artifact_family: String,
    },
    PluginCompatibility(PluginCompatibilitySummary),
    CachePolicy(CachePolicySummary),
    ContractCompatibilityGate(CompatibilityGateSummary),
    ReleaseReadiness(ReleaseReadinessSummary),
    BackendRuntimePosture(BackendRuntimePostureSummary),
    BackendAdmissibility {
        admissibility: BackendAdmissibility,
        reason: String,
    },
    CudaBackendPromotion(CudaBackendPromotionSummary),
}

impl ComputeEvidenceSummary {
    pub fn evidence_kind(&self) -> ComputeEvidenceKind {
        match self {
            Self::GenericArtifact { .. } => ComputeEvidenceKind::ComputeArtifact,
            Self::PluginCompatibility(_) => ComputeEvidenceKind::PluginCompatibility,
            Self::CachePolicy(_) => ComputeEvidenceKind::CachePolicy,
            Self::ContractCompatibilityGate(_) => ComputeEvidenceKind::ContractCompatibilityGate,
            Self::ReleaseReadiness(_) => ComputeEvidenceKind::ReleaseReadiness,
            Self::BackendRuntimePosture(_) => ComputeEvidenceKind::BackendRuntimePosture,
            Self::BackendAdmissibility { .. } => ComputeEvidenceKind::BackendAdmissibility,
            Self::CudaBackendPromotion(_) => ComputeEvidenceKind::CudaBackendPromotion,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendAdmissibility {
    Admissible,
    Inadmissible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCompatibility {
    Compatible,
    Incompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCompatibilityReason {
    DeclaredCompatible,
    PluginKeyMismatch,
    BoundaryKindMismatch,
    OperationKindMismatch,
    BackendCapabilityInadmissible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginCompatibilitySummary {
    pub plugin_key: String,
    pub boundary_kind: String,
    pub operation_kind: String,
    pub compatibility: PluginCompatibility,
    pub reason: PluginCompatibilityReason,
    pub backend_admissibility: Option<BackendAdmissibility>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLifecycleState {
    Fresh,
    Stale,
    Invalid,
    Retired,
    BlockedDependencyMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheReuseAdmissibility {
    ReuseAdmissible,
    ReuseRefused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheCoherencePosture {
    Coherent,
    DependencyMissing,
    DependencyContentDrift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecomputeReason {
    DirectArtifactChanged,
    UpstreamDependencyChanged,
    DependencyContentDrift,
    ArtifactRetired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheBlockedReason {
    DependencyMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePolicySummary {
    pub lifecycle_state: CacheLifecycleState,
    pub reuse_admissibility: CacheReuseAdmissibility,
    pub recompute_reason: Option<RecomputeReason>,
    pub blocked_reason: Option<CacheBlockedReason>,
    pub coherence_posture: CacheCoherencePosture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityClassification {
    Additive,
    CompatibleTightening,
    Breaking,
    InternalOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityGateSummary {
    pub classification: CompatibilityClassification,
    pub gate_blocking: bool,
    pub version_bump_required: bool,
    pub readiness_bump_required: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceReadiness {
    Ready,
    NeedsReview,
    Blocked,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseReadinessSummary {
    pub readiness: EvidenceReadiness,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendRole {
    Reference,
    Optimized,
    Spectral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMemoryLayoutPosture {
    HostCanonical,
    DeviceShadowVersioned,
    VersionMismatch,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionPosture {
    DeterministicReference,
    ExplicitPolicy,
    Mismatch,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendCanonicalizationPosture {
    BackendIndependent,
    BackendLocalOnly,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendParityOracle {
    CpuReference,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRuntimePostureSummary {
    pub backend_role: BackendRole,
    pub layout_version: String,
    pub layout_posture: BackendMemoryLayoutPosture,
    pub precision_posture: PrecisionPosture,
    pub canonicalization_posture: BackendCanonicalizationPosture,
    pub parity_oracle: BackendParityOracle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendRuntimeTrack {
    CpuReference,
    CudaOptimized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendLayoutVersion {
    HostCanonicalV0,
    CudaDeviceTensorV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostDeviceTransferSemantics {
    HostLocal,
    HostDeviceRoundTrip,
    DeviceOnlyNoCanonicalReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendPrecisionMode {
    DeterministicReference,
    Float32Deterministic,
    Float64Deterministic,
    MixedPrecisionExplicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendPackingPolicy {
    CanonicalHost,
    DeviceLocalContiguous,
    ExplicitTensorPacked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendLayoutCompatibility {
    Compatible,
    Canonicalizable,
    ReviewRequired,
    Breaking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaParityBudget {
    Exact,
    BoundedUnits { max_delta_units: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaCanonicalizationPosture {
    Canonicalized,
    RematerializationRequired,
    NonCanonicalizable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaWorkloadEligibility {
    Eligible,
    Ineligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaWorkloadEligibilityReason {
    EngineWorkloadEligible,
    BackendInadmissible,
    RuntimeTrackNotCuda,
    WorkloadNotCudaEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaParityStatus {
    ParityClean,
    ParityWithinBudget,
    ParityOverBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaPromotionPosture {
    Promote,
    Hold,
    Degrade,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaPromotionReason {
    PromotionEligible,
    ParityWithinBudget,
    BackendInadmissible,
    WorkloadIneligible,
    LayoutReviewRequired,
    LayoutBreaking,
    CanonicalizationRequired,
    NonCanonicalizable,
    ParityOverBudget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CudaBackendPromotionSummary {
    pub backend_admissibility: BackendAdmissibility,
    pub runtime_track: BackendRuntimeTrack,
    pub layout_version: BackendLayoutVersion,
    pub transfer_semantics: HostDeviceTransferSemantics,
    pub precision_mode: BackendPrecisionMode,
    pub packing_policy: BackendPackingPolicy,
    pub canonicalization_boundary: String,
    pub layout_compatibility: BackendLayoutCompatibility,
    pub parity_budget: CudaParityBudget,
    pub observed_delta_units: u64,
    pub canonicalization: CudaCanonicalizationPosture,
    pub workload_eligibility: CudaWorkloadEligibility,
    pub eligibility_reason: CudaWorkloadEligibilityReason,
    pub parity_status: CudaParityStatus,
    pub promotion_posture: CudaPromotionPosture,
    pub promotion_reason: CudaPromotionReason,
}

pub fn is_strict_content_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}
