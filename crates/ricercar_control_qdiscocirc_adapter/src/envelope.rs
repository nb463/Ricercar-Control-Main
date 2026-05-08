use crate::model::{
    ProbeChange, ProbeEdge, ProbeExplanation, ProbeNode, ProbeTrace, ProbeWalkthroughKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeEnvelope {
    pub envelope_id: String,
    pub title: String,
    pub nodes: Vec<ProbeNode>,
    pub edges: Vec<ProbeEdge>,
    pub traces: Vec<ProbeTrace>,
    pub explanation: ProbeExplanation,
}

pub fn probe_envelope(
    envelope_id: impl Into<String>,
    title: impl Into<String>,
    walkthrough: ProbeWalkthroughKind,
    summary: impl Into<String>,
) -> ProbeEnvelopeBuilder {
    ProbeEnvelopeBuilder {
        envelope_id: envelope_id.into(),
        title: title.into(),
        nodes: Vec::new(),
        edges: Vec::new(),
        traces: Vec::new(),
        explanation: ProbeExplanation {
            walkthrough,
            summary: summary.into(),
            changed: Vec::new(),
            blocking_reason_ids: Vec::new(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeEnvelopeBuilder {
    envelope_id: String,
    title: String,
    nodes: Vec<ProbeNode>,
    edges: Vec<ProbeEdge>,
    traces: Vec<ProbeTrace>,
    explanation: ProbeExplanation,
}

impl ProbeEnvelopeBuilder {
    pub fn node(mut self, node: ProbeNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn edge(mut self, edge: ProbeEdge) -> Self {
        self.edges.push(edge);
        self
    }

    pub fn trace(mut self, trace: ProbeTrace) -> Self {
        self.traces.push(trace);
        self
    }

    pub fn blocking_reason(mut self, reason_id: impl Into<String>) -> Self {
        self.explanation.blocking_reason_ids.push(reason_id.into());
        self
    }

    pub fn change(mut self, change: ProbeChange) -> Self {
        self.explanation.changed.push(change);
        self
    }

    pub fn build(self) -> ProbeEnvelope {
        ProbeEnvelope {
            envelope_id: self.envelope_id,
            title: self.title,
            nodes: self.nodes,
            edges: self.edges,
            traces: self.traces,
            explanation: self.explanation,
        }
    }
}
