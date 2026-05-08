use crate::{
    envelope::probe_envelope,
    model::{
        ProbeChange, ProbeDeltaCause, ProbeDeltaSummary, ProbeEdge, ProbeEdgeKind, ProbeNode,
        ProbeNodeRole, ProbeTrace, ProbeWalkthroughKind,
    },
    refs::SourceRef,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RolePresence {
    saw_compute_truth: bool,
    saw_control_truth: bool,
    saw_probe_only: bool,
}

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

    for source_id in all_source_ids(before, after) {
        if node_signatures_for_source_id(before, &source_id)
            != node_signatures_for_source_id(after, &source_id)
        {
            builder = builder.change(ProbeChange {
                source_id: source_id.clone(),
                before: change_snapshot_for_source_id(before, &source_id),
                after: change_snapshot_for_source_id(after, &source_id),
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
        .collect::<BTreeSet<_>>();
    let mut compute_truth_refs = BTreeSet::new();
    let mut control_consequence_refs = BTreeSet::new();
    let mut saw_probe_only = false;

    for source_id in &changed_source_ids {
        let before_roles = role_presence_for_source_id(before, source_id);
        let after_roles = role_presence_for_source_id(after, source_id);

        if before_roles.saw_compute_truth || after_roles.saw_compute_truth {
            compute_truth_refs.insert(source_id.clone());
        }
        if before_roles.saw_control_truth || after_roles.saw_control_truth {
            control_consequence_refs.insert(source_id.clone());
        }
        if before_roles.saw_probe_only || after_roles.saw_probe_only {
            saw_probe_only = true;
        }
    }

    let cause = match (
        compute_truth_refs.is_empty(),
        control_consequence_refs.is_empty(),
        saw_probe_only,
    ) {
        (true, true, false) => {
            debug_assert!(
                changed_source_ids.is_empty(),
                "NoChange reached with changed source ids present"
            );
            ProbeDeltaCause::NoChange
        }
        (false, true, false) => ProbeDeltaCause::ComputeTruthChanged,
        (true, false, false) => ProbeDeltaCause::ControlConsequenceChanged,
        (true, true, true) => ProbeDeltaCause::ProbeOnlyChanged,
        _ => ProbeDeltaCause::MixedOwnershipChanged,
    };

    ProbeDeltaSummary {
        cause,
        changed_source_ids: changed_source_ids.into_iter().collect(),
        compute_truth_refs: compute_truth_refs.into_iter().collect(),
        control_consequence_refs: control_consequence_refs.into_iter().collect(),
    }
}

fn all_source_ids(
    before: &crate::envelope::ProbeEnvelope,
    after: &crate::envelope::ProbeEnvelope,
) -> BTreeSet<String> {
    before
        .nodes
        .iter()
        .chain(&after.nodes)
        .map(|node| node.source.source_id.clone())
        .collect()
}

fn node_signatures_for_source_id(
    envelope: &crate::envelope::ProbeEnvelope,
    source_id: &str,
) -> Vec<String> {
    let mut signatures = envelope
        .nodes
        .iter()
        .filter(|node| node.source.source_id == source_id)
        .map(node_signature)
        .collect::<Vec<_>>();
    signatures.sort();
    signatures.dedup();
    signatures
}

fn change_snapshot_for_source_id(
    envelope: &crate::envelope::ProbeEnvelope,
    source_id: &str,
) -> String {
    let signatures = node_signatures_for_source_id(envelope, source_id);
    if signatures.is_empty() {
        "missing".to_string()
    } else {
        signatures.join(" || ")
    }
}

fn node_signature(node: &ProbeNode) -> String {
    let mut tags = node.tags.clone();
    tags.sort();
    tags.dedup();
    format!(
        "role={} summary={} tags={}",
        role_id(node.role),
        node.summary,
        tags.join(",")
    )
}

fn role_presence_for_source_id(
    envelope: &crate::envelope::ProbeEnvelope,
    source_id: &str,
) -> RolePresence {
    let mut presence = RolePresence::default();
    for role in envelope
        .nodes
        .iter()
        .filter(|node| node.source.source_id == source_id)
        .map(|node| node.role)
    {
        match role {
            ProbeNodeRole::ComputeTruth => presence.saw_compute_truth = true,
            ProbeNodeRole::ControlTruth => presence.saw_control_truth = true,
            ProbeNodeRole::ProbeOnly => presence.saw_probe_only = true,
        }
    }
    presence
}

fn role_id(role: ProbeNodeRole) -> &'static str {
    match role {
        ProbeNodeRole::ComputeTruth => "compute_truth",
        ProbeNodeRole::ControlTruth => "control_truth",
        ProbeNodeRole::ProbeOnly => "probe_only",
    }
}
