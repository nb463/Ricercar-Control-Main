use crate::model::SourcePlane;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRef {
    pub source_plane: SourcePlane,
    pub source_kind: String,
    pub source_id: String,
    pub content_hash: Option<String>,
    pub replay_ref: Option<String>,
    pub lineage: Vec<String>,
    pub compute_owned: bool,
}

impl SourceRef {
    pub fn control_truth(
        source_kind: impl Into<String>,
        source_id: impl Into<String>,
        replay_ref: Option<String>,
        lineage: Vec<String>,
    ) -> Self {
        Self {
            source_plane: SourcePlane::Control,
            source_kind: source_kind.into(),
            source_id: source_id.into(),
            content_hash: None,
            replay_ref,
            lineage,
            compute_owned: false,
        }
    }

    pub fn compute_evidence(
        source_kind: impl Into<String>,
        source_id: impl Into<String>,
        content_hash: Option<String>,
        replay_ref: Option<String>,
        lineage: Vec<String>,
    ) -> Self {
        Self {
            source_plane: SourcePlane::Compute,
            source_kind: source_kind.into(),
            source_id: source_id.into(),
            content_hash,
            replay_ref,
            lineage,
            compute_owned: true,
        }
    }

    pub fn probe_only(source_kind: impl Into<String>, source_id: impl Into<String>) -> Self {
        Self {
            source_plane: SourcePlane::Probe,
            source_kind: source_kind.into(),
            source_id: source_id.into(),
            content_hash: None,
            replay_ref: None,
            lineage: Vec::new(),
            compute_owned: false,
        }
    }
}
