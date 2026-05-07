# Glossary

## Ricercar-Compute

The compute plane and kernel family. It owns derived compute truth: semantic artifacts, lawful construction, stable-structure extraction, observables, residuals, repair evidence, and replay-visible lineage.

## Ricercar-Control

The decision-core and workflow-truth layer. It owns artifact intake posture, trust and disposition state, explanation policy, orchestration posture, assignments, approvals, escalation or suppression choices, pursuit decisions, and workflow audit truth.

## Product / App Layer

The user-facing and business-action layer. It prepares and routes domain material, presents UX, performs product-specific actions, and integrates with external business systems.

## Compute Artifact

A replayable, validated output from Ricercar-Compute. It can inform workflow decisions but is not workflow truth by itself.

## Workflow Truth

The operational record of what the workflow decided or did. It includes assignment, approval, escalation, suppression, pursuit, review, explanation, and final audit state.

## Artifact Intake

The Control-owned process by which compute artifacts become eligible to inform workflow state. Intake is not automatic promotion.

## Intake Boundary

The Control-owned boundary where compute artifacts are checked for identity, provenance, replayability, lineage, freshness, and workflow fit before they may influence governance.

## Admission

The Control-owned boundary judgment that a compute artifact has enough identity, provenance, replayability, lineage, freshness, and semantic validity to become eligible for governance. Admission is not approval.

## Admission Envelope

A Control-side wrapper around compute evidence, workflow context, provenance, replay references, validation posture, semantic posture, and typed evidence summary. PR D introduces the first narrow implementation.

## Admission Record

The Control-owned record of whether an admission envelope was admitted or rejected, including explicit rejection reasons when intake fails.

## Provenance

The evidence of where an admitted artifact came from, including its source system, compute artifact identity, workflow context, and boundary path into Control.

## Lineage

The chain of source references, updates, derivations, or prior artifacts needed to explain why admitted evidence exists and what it depends on.

## Replayability

The expectation that admitted compute evidence carries enough replay reference, content identity, or validation posture for Control to audit how the evidence was produced or why it was accepted.

## Trust Class

A Control-owned interpretation of admitted evidence for workflow purposes, such as ready, review-required, degraded, fallback-only, or refused. Trust classes do not change the underlying compute semantics.

## Decision-Core Truth

The Control-owned truth of decisions, dispositions, and workflow state transitions. It is informed by evidence but must remain distinct from raw compute artifact truth.

## Disposition

A Control-owned workflow judgment about how admitted compute evidence should be handled operationally, such as promotion, fallback, refusal, escalation, suppression, or review posture.

## Promotion

A Control-owned disposition that allows admitted evidence to advance a workflow state or candidate action. Promotion is not automatic admission.

## Fallback

A Control-owned disposition that limits admitted evidence to secondary or fallback use when ordinary promotion is not justified.

## Refusal

A Control-owned disposition that prevents admitted evidence from influencing workflow consequence.

## Suppress

A Control-owned disposition that intentionally withholds admitted evidence from ordinary surfacing, routing, or action flow while preserving an auditable governance outcome. Suppression does not mean the evidence is compute-invalid.

## Degrade

A Control-owned disposition that carries admitted evidence forward with reduced operational standing, confidence, or allowed consequence.

## Escalate

A Control-owned disposition that routes admitted evidence to higher scrutiny, review, or governance attention before consequence.

## HoldForReview

A Control-owned disposition that preserves admitted evidence without allowing workflow consequence until human or policy review occurs. It is not refusal; it is an explicit suspension pending review.

## Governance Outcome

The explicit Control-owned result of applying disposition doctrine to admitted evidence. Core outcomes include promotion, fallback, refusal, suppression, degrade, escalation, and hold for review.

## Control Trace

A workflow-context-scoped collection of admission envelopes assembled for Control interpretation. PR D keeps traces local and deterministic; they are not schedulers or orchestration state.

## Queueable Work Item

A Control-owned routing input that groups admitted-or-admittable compute evidence with a workflow context, routing intent, and queue priority.

## Routing Intent

A Control-owned declaration of the requested routing posture, such as normal execution, accelerated execution, fallback execution, recompute, suppression, or review.

## Routing Decision

The Control-owned decision that maps admitted evidence and routing intent to a concrete execution command with replayable routing reasons.

## Execution Command

A Control-owned routing consequence such as promote for execution, hold for review, escalate, refuse execution, trigger recompute, route to fallback, or suppress ordinary routing.

## Orchestration Audit Record

The workflow-auditable record of a routing decision, execution command, orchestration state, explanation bundle, and routing explanation payload.

## Routing Explanation Payload

The structured explanation emitted for a consequential routing act. It preserves admitted and rejected evidence keys, Compute-owned reason ids, Control-owned routing reasons, and an audit reference.

## Explanation Bundle

A workflow-auditable Control output that records admission results, trust class, disposition, incident fragments, surfacing directives, and a compact audit summary.

## Surfacing Grammar

The Control-owned vocabulary for how an explanation bundle should be exposed to operators, downstream systems, and audit consumers. It is not product UX and does not perform routing.

## Incident Taxonomy

The typed set of explanation incident kinds used to distinguish intake rejection, degraded evidence, plugin incompatibility, cache posture, boundary drift, readiness blocking, backend parity review, and ready evidence.

## Diagram Hint

A lightweight optional explanation surface that names wires, boxes, traces, contractions, and splits for tensor/string-diagram-style narration. It is explanatory only and does not add compute semantics.

## Truth Non-Bleed

The doctrine that compute truth, workflow truth, and product action must remain distinct unless a boundary crossing is explicit, auditable, and explainable.

## Derived Structural Intelligence

Ricercar's shared capability for turning partial, contradictory, or early evidence into evidence-accountable structure that can support human decision-making without black-boxing the reasoning process.
