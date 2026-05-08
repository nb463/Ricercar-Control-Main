use crate::{
    envelope::ProbeEnvelope,
    model::{
        operator_probe_question_id, probe_walkthrough_kind_id, OperatorProbeQuestion,
        OperatorProbeView, ProbeDeltaSummary, ProbeNodeRole,
    },
};

pub fn stable_probe_summary(envelope: &ProbeEnvelope) -> String {
    let mut lines = Vec::new();
    lines.push(format!("envelope|{}", envelope.envelope_id));
    lines.push(format!("title|{}", envelope.title));
    lines.push(format!(
        "walkthrough|{}",
        probe_walkthrough_kind_id(envelope.explanation.walkthrough)
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

pub fn operator_view_for_probe(
    scenario_key: impl Into<String>,
    question: OperatorProbeQuestion,
    envelope: &ProbeEnvelope,
) -> OperatorProbeView {
    let compute_truth_refs = envelope
        .nodes
        .iter()
        .filter(|node| node.role == ProbeNodeRole::ComputeTruth)
        .map(|node| node.source.source_id.clone())
        .collect::<Vec<_>>();
    let control_consequence_refs = envelope
        .nodes
        .iter()
        .filter(|node| node.role == ProbeNodeRole::ControlTruth)
        .map(|node| node.source.source_id.clone())
        .collect::<Vec<_>>();
    OperatorProbeView {
        scenario_key: scenario_key.into(),
        question,
        answer_summary: format!(
            "question={} envelope={} blocked={} changes={}",
            operator_probe_question_id(question),
            envelope.envelope_id,
            envelope.explanation.blocking_reason_ids.join(","),
            envelope.explanation.changed.len()
        ),
        compute_truth_refs,
        control_consequence_refs,
        probe_envelope_refs: vec![envelope.envelope_id.clone()],
    }
}

pub fn stable_operator_view_summary(view: &OperatorProbeView) -> String {
    let mut lines = Vec::new();
    lines.push(format!("scenario|{}", view.scenario_key));
    lines.push(format!(
        "question|{}",
        operator_probe_question_id(view.question)
    ));
    lines.push(format!("answer|{}", view.answer_summary));
    for reference in &view.compute_truth_refs {
        lines.push(format!("compute_truth_ref|{reference}"));
    }
    for reference in &view.control_consequence_refs {
        lines.push(format!("control_consequence_ref|{reference}"));
    }
    for reference in &view.probe_envelope_refs {
        lines.push(format!("probe_ref|{reference}"));
    }
    lines.join("\n")
}

pub fn stable_delta_summary(summary: &ProbeDeltaSummary) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "delta_cause|{}",
        crate::model::probe_delta_cause_id(summary.cause)
    ));
    for source_id in &summary.changed_source_ids {
        lines.push(format!("changed_ref|{source_id}"));
    }
    for reference in &summary.compute_truth_refs {
        lines.push(format!("compute_truth_ref|{reference}"));
    }
    for reference in &summary.control_consequence_refs {
        lines.push(format!("control_consequence_ref|{reference}"));
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
