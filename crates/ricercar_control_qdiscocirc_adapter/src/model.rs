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
