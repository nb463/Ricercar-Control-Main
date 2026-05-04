# Control Admission And Disposition Boundary v0

## Purpose

PR B/C defines the first Control-side boundary from compute artifact intake to explicit governance outcome.

This is a doctrine/specification slice. It does not implement a runtime intake service, policy engine, scheduler, artifact store, orchestrator, or product workflow.

The boundary exists so Ricercar-Control can answer:

- whether a compute artifact is admitted or rejected at intake
- what Control-owned trust meaning can be attached after admission
- what disposition or governance outcome may follow
- why workflow consequence remains explicit rather than implied by compute output

## Boundary Ownership

Ricercar-Compute owns derived compute truth. It produces replayable semantic artifacts, comparison evidence, residuals, repair reports, update traces, task results, backend admissibility evidence, and related compute outputs.

Ricercar-Control owns artifact intake truth, trust interpretation, governance, workflow consequence, and operational meaning.

Products own concrete UX and business action surfaces.

Compute artifacts may inform workflow truth. They must never silently become workflow truth.

## Admission Boundary

A Control admission boundary receives compute artifacts and evaluates whether they are eligible to inform workflow state.

Admission must require, at minimum:

- artifact identity
- provenance and source context
- replay reference or replayability claim
- lineage sufficient to explain where the artifact came from
- validation posture from the compute boundary
- freshness or update posture when relevant
- semantic admissibility for the workflow context

Control must reject artifacts before governance begins when they are:

- malformed
- semantically invalid
- stale relative to required workflow context
- missing required provenance, replay, or lineage
- incomplete for the claimed workflow use
- non-admissible under Control intake requirements
- contradictory in ways that cannot be reviewed or explained

Rejection at admission is a Control-owned intake outcome. It is not a recomputation of compute semantics.

The sequence is admission or rejection first, trust classification only for admitted evidence, and then an explicit disposition or governance outcome.

## Admission Is Not Approval

An admitted artifact is eligible to inform governance. It is not automatically approved for workflow consequence.

Admission does not create:

- assignment
- approval
- promotion
- fallback
- suppression
- escalation
- pursuit
- final audit state

Those require explicit Control-owned disposition.

## Trust Classes

A trust class is a Control-owned interpretation of admitted compute evidence for workflow purposes.

Trust classes should remain separate from compute semantics. For example, a compute artifact may be mathematically lawful but still require review in a sensitive workflow. A degraded artifact may be useful for fallback or escalation but not promotion.

Initial trust classes are:

- `Ready`: admitted evidence can support ordinary workflow consideration.
- `ReviewRequired`: admitted evidence may be useful but needs human or policy review before consequence.
- `Degraded`: admitted evidence is interpretable but has reduced confidence, fidelity, freshness, or workflow fit.
- `FallbackOnly`: admitted evidence may support fallback posture but not promotion.
- `Refused`: evidence is not eligible for workflow consequence.

These are Control judgments. They do not rewrite compute classifications.

## Disposition And Governance Outcomes

Disposition is the Control-owned judgment about how admitted evidence should be handled operationally.

Initial governance outcomes are:

- `Promote`: allow the evidence to advance a workflow state or candidate action.
- `Fallback`: use the evidence only as fallback or secondary support.
- `Refuse`: prevent evidence from influencing workflow consequence because it is not eligible under Control governance.
- `Suppress`: intentionally withhold admitted evidence from ordinary surfacing, routing, or action flow while preserving an auditable Control-owned governance outcome.
- `Degrade`: carry the evidence forward with reduced operational standing.
- `Escalate`: route the evidence to a higher scrutiny or review posture.
- `HoldForReview`: preserve the evidence without allowing consequence until review occurs.

Every governance outcome must be explicit, auditable, and explainable.

## Comparison-Aware Governance

Comparison results from Compute may inform Control trust and disposition.

Control may use comparison evidence to decide whether to promote, degrade, fallback, refuse, suppress, escalate, or hold for review.

Control must not reimplement comparison computation or silently reinterpret comparison semantics. It governs the workflow meaning of comparison evidence.

## Repair Acceptance Governance

Repair approximation reports from Compute may inform Control trust and disposition.

Control may accept repair evidence for ordinary workflow use, degrade it, limit it to fallback, refuse it, suppress it, escalate it, or hold it for review.

Control must not reimplement repair math. Repair acceptance is governance over repair evidence, not the repair computation itself.

## Workflow Consequence

Workflow consequence is Control-owned.

An admitted compute artifact may influence workflow state only through explicit Control state transition, disposition, audit, or review posture.

Products may execute concrete business actions only after Control has supplied the decision-core state required by that workflow.

## Non-Scope

PR B/C does not add:

- compute kernels
- compute artifact semantic validation beyond consuming Compute's boundary posture
- backend implementation
- runtime intake validators
- policy engine implementation
- scheduling or orchestration runtime
- global artifact store
- product-specific UX or workflow screens
- explanation grammar beyond these boundary references
