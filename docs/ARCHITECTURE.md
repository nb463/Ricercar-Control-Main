# Ricercar-Control Architecture

Ricercar-Control is the Compute Plane consumer and workflow-truth owner for Ricercar systems. It sits between Ricercar-Compute and product/app layers.

## Layer Split

### Ricercar-Compute

Ricercar-Compute owns derived compute truth.

It produces replayable, fail-closed semantic artifacts such as witnessed interactions, representables, observables, task results, update traces, comparison results, residual traces, repair approximation reports, completion/cocompletion outputs, and other lawful kernel products.

Compute answers questions like:

- What structure is supported by the evidence?
- What changed?
- What was retained or lost?
- What is stable, degraded, refused, or non-comparable?
- What can be replayed and audited?

Compute does not own workflow consequences.

### Ricercar-Control

Ricercar-Control owns workflow truth and decision-core truth.

It decides how compute artifacts are admitted, interpreted, surfaced, explained, escalated, suppressed, assigned, or carried into human decision workflows. Control preserves operational state and auditability for workflow decisions.

Control answers questions like:

- Should this artifact enter the workflow?
- Who needs to review it?
- What disposition does it receive?
- What explanation is appropriate for this context?
- What is escalated, suppressed, pursued, deferred, or rejected?
- What did humans and systems actually decide?

Control does not reimplement compute kernels.

### Product and App Layers

Products and apps own concrete business action surfaces.

They prepare domain material, call Control and Compute through proper boundaries, present UX, perform product-specific actions, and integrate with customer or operator systems.

Products answer questions like:

- What does this look like to a user?
- Which business object is updated?
- Which customer-facing action is taken?
- Which product-specific workflow is completed?

Products do not become the shared decision-core.

## Artifact Intake Doctrine

Compute artifacts are evidence-bearing inputs to Control. They are not workflow truth by themselves.

Control intake must evaluate:

- artifact identity and provenance
- source system and workflow context
- replay and validation status
- exact, degraded, refused, or non-comparable semantics
- freshness and update lineage
- whether the artifact is eligible for surfacing or action
- what human review, explanation, or escalation is required

PR B/C defines the first admission boundary around these expectations. It says malformed, stale, incomplete, or semantically invalid compute output must be rejected before governance begins. It does not implement validators or runtime intake.

Admission is not approval. An admitted artifact is only eligible to inform Control-owned trust and disposition judgments.

PR D adds the first narrow implementation of this boundary as typed admission envelopes and admission records. It consumes compute evidence summaries and validation posture supplied by Compute; it does not recompute artifact semantics.

## Trust And Disposition Doctrine

Control-owned trust is an operational interpretation of admitted compute evidence. It is not mathematical truth and must not rewrite compute semantics.

Trust classes should remain explicit enough to distinguish evidence that is ready for ordinary workflow use, evidence that needs review, evidence that is degraded but still interpretable, evidence that is only useful as fallback context, and evidence that must be refused.

Disposition is the Control-owned judgment about what happens next. The core disposition vocabulary includes:

- promotion
- fallback
- refusal
- suppression
- degrade
- escalation
- hold for review

These outcomes are workflow meanings. Compute may supply comparison results, residuals, repair artifacts, updates, backend admissibility evidence, or task results, but Control owns the governance decision about how those artifacts affect workflow state.

Comparison-aware governance means Control interprets comparison evidence for workflow consequence. It does not recompute comparison semantics.

Repair acceptance means Control governs whether repair evidence is admissible for workflow use, fallback, refusal, suppression, escalation, or review. It does not perform repair math.

## Explanation And Surfacing Doctrine

Control explanations are workflow-auditable assemblies over admitted or rejected evidence.

PR D introduces explanation bundles that preserve:

- admission outcome and rejection reasons
- Control-owned trust class
- Control-owned disposition
- incident/debug taxonomy
- operator, downstream-system, and audit surfacing directives
- a lightweight diagram hint vocabulary for explanation surfaces only

The surfacing grammar is not product UX and not a runtime router. It tells downstream systems how Control has interpreted evidence for workflow posture. Products still decide concrete screens, labels, and business actions.

The evidence chain remains explicit:

1. backend capability
2. plugin compatibility
3. cache/recompute posture
4. compatibility verdict
5. release/readiness posture

Backend runtime and CUDA-transition evidence are interpreted as Control posture only after Compute emits typed compatibility/readiness signals. CPU/reference semantics remain the canonical oracle. Runtime memory layout and transfer details remain backend-local implementation evidence; they do not become public compute artifact truth.

## Workflow Truth Doctrine

Workflow truth is the record of operational decision state.

It includes assignments, approvals, dispositions, suppression and escalation choices, pursuit decisions, human review state, explanation state, and final audit history. It can be informed by compute artifacts, but it must be explicitly owned and recorded by Control.

A compute artifact can say "this evidence supports a degraded but interpretable residual." It cannot silently say "approve this claim", "assign this person", "suppress this case", or "pursue this opportunity." Those are Control or product-layer decisions.

## Truth Non-Bleed Rules

1. Compute truth must not silently become workflow truth.
2. Workflow truth must not masquerade as compute truth.
3. Product action must not bypass Control when a decision-core state transition is required.
4. Control must not reimplement kernel math that belongs in Compute.
5. Products must not become the shared policy or disposition layer.
6. Every movement from artifact evidence to workflow state must be explicit, auditable, and explainable.

## Why This Boundary Matters

Ricercar is valuable because it can help systems scale without going mute: abstractions can compress without becoming dishonest, updates can be narrated instead of merely endured, structure can be inspected instead of merely consumed, and outputs can be defensible because their semantic lineage remains visible.

Control is where those properties become operationally trustworthy. It protects the path from evidence to human action without hiding the difference between "the math says this" and "the workflow decided this."
