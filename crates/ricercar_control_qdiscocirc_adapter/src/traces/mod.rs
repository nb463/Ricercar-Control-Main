use crate::{
    envelope::probe_envelope,
    model::{
        ProbeChange, ProbeEdge, ProbeEdgeKind, ProbeNode, ProbeNodeRole, ProbeTrace,
        ProbeWalkthroughKind,
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
