use crate::{
    envelope::probe_envelope,
    model::{
        ProbeChange, ProbeDeltaCause, ProbeDeltaSummary, ProbeEdge, ProbeEdgeKind, ProbeNode,
        ProbeNodeRole, ProbeTrace, ProbeWalkthroughKind,
    },
    refs::SourceRef,
};

pub fn compare_probe_envelopes(
    before: &crate::envelope::ProbeEnvelope,
    after: &crate::envelope::ProbeEnvelope,
) -> crate::envelope::ProbeEnvelope {
    let mut builder = probe_envelope(
        format!("probe/diff/{}/{}", before.envelope_id, after.envelope_id),
        "Show me what changed",
        ProbeWalkthroughKind::ShowMeWhatChanged,
        "typed probe diff over existing Control probe fields",
    )
    .node(ProbeNode::new(
        "before",
        ProbeNodeRole::ProbeOnly,
        SourceRef::probe_only("probe_envelope", &before.envelope_id),
        "before envelope",
        before.title.clone(),
        vec!["before".to_string()],
    ))
    .node(ProbeNode::new(
        "after",
        ProbeNodeRole::ProbeOnly,
        SourceRef::probe_only("probe_envelope", &after.envelope_id),
        "after envelope",
        after.title.clone(),
        vec!["after".to_string()],
    ))
    .edge(ProbeEdge::new(
        "before",
        "after",
        ProbeEdgeKind::Changes,
        "changed",
    ));

    for after_node in &after.nodes {
        let before_node = before
            .nodes
            .iter()
            .find(|node| node.source.source_id == after_node.source.source_id);
        match before_node {
            Some(before_node)
                if before_node.summary != after_node.summary
                    || before_node.tags != after_node.tags =>
            {
                builder = builder.change(ProbeChange {
                    source_id: after_node.source.source_id.clone(),
                    before: before_node.summary.clone(),
                    after: after_node.summary.clone(),
                });
            }
            None => {
                builder = builder.change(ProbeChange {
                    source_id: after_node.source.source_id.clone(),
                    before: "missing".to_string(),
                    after: after_node.summary.clone(),
                });
            }
            _ => {}
        }
    }
    for before_node in &before.nodes {
        if !after
            .nodes
            .iter()
            .any(|node| node.source.source_id == before_node.source.source_id)
        {
            builder = builder.change(ProbeChange {
                source_id: before_node.source.source_id.clone(),
                before: before_node.summary.clone(),
                after: "missing".to_string(),
            });
        }
    }

    builder
        .trace(ProbeTrace {
            trace_id: "trace/diff".to_string(),
            title: "Control probe diff".to_string(),
            node_ids: vec!["before".to_string(), "after".to_string()],
            summary: "no workflow-policy diffing; only typed probe fields are compared".to_string(),
        })
        .build()
}

pub fn summarize_probe_delta(
    before: &crate::envelope::ProbeEnvelope,
    after: &crate::envelope::ProbeEnvelope,
) -> ProbeDeltaSummary {
    let diff = compare_probe_envelopes(before, after);
    let changed_source_ids = diff
        .explanation
        .changed
        .iter()
        .map(|change| change.source_id.clone())
        .collect::<Vec<_>>();
    let mut compute_truth_refs = Vec::new();
    let mut control_consequence_refs = Vec::new();
    let mut saw_probe_only = false;

    for source_id in &changed_source_ids {
        let node = after
            .nodes
            .iter()
            .chain(&before.nodes)
            .find(|node| node.source.source_id == *source_id);
        match node.map(|node| node.role) {
            Some(ProbeNodeRole::ComputeTruth) => compute_truth_refs.push(source_id.clone()),
            Some(ProbeNodeRole::ControlTruth) => control_consequence_refs.push(source_id.clone()),
            Some(ProbeNodeRole::ProbeOnly) | None => saw_probe_only = true,
        }
    }

    let cause = match (
        compute_truth_refs.is_empty(),
        control_consequence_refs.is_empty(),
        saw_probe_only,
    ) {
        (true, true, false) => ProbeDeltaCause::NoChange,
        (false, true, false) => ProbeDeltaCause::ComputeTruthChanged,
        (true, false, false) => ProbeDeltaCause::ControlConsequenceChanged,
        (true, true, true) => ProbeDeltaCause::ProbeOnlyChanged,
        _ => ProbeDeltaCause::MixedOwnershipChanged,
    };

    ProbeDeltaSummary {
        cause,
        changed_source_ids,
        compute_truth_refs,
        control_consequence_refs,
    }
}
