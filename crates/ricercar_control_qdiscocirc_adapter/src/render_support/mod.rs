use crate::{envelope::ProbeEnvelope, model::ProbeNodeRole};

pub fn stable_probe_summary(envelope: &ProbeEnvelope) -> String {
    let mut lines = Vec::new();
    lines.push(format!("envelope|{}", envelope.envelope_id));
    lines.push(format!("title|{}", envelope.title));
    lines.push(format!(
        "walkthrough|{:?}",
        envelope.explanation.walkthrough
    ));
    lines.push(format!("summary|{}", envelope.explanation.summary));
    for node in &envelope.nodes {
        lines.push(format!(
            "node|{}|{}|{}|{}",
            node.node_id,
            role_id(node.role),
            node.source.source_kind,
            node.summary
        ));
    }
    for reason in &envelope.explanation.blocking_reason_ids {
        lines.push(format!("blocked|{reason}"));
    }
    for change in &envelope.explanation.changed {
        lines.push(format!(
            "changed|{}|{}|{}",
            change.source_id, change.before, change.after
        ));
    }
    lines.join("\n")
}

fn role_id(role: ProbeNodeRole) -> &'static str {
    match role {
        ProbeNodeRole::ComputeTruth => "compute_truth",
        ProbeNodeRole::ControlTruth => "control_truth",
        ProbeNodeRole::ProbeOnly => "probe_only",
    }
}
