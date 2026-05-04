# Ricercar-Control Roadmap

Ricercar-Control starts after the current Ricercar-Compute artifact-boundary and local-cache/recompute tranches, with constitutional boundaries before runtime machinery.

## PR A: Constitutional Skeleton And Boundary Ownership

Status: foundation.

PR A establishes:

- Ricercar-Control as decision-core and workflow-truth owner
- Ricercar-Compute as math engine, compute plane, and kernel family
- product/app layers as UX and concrete business action owners
- artifact intake doctrine
- workflow truth doctrine
- truth non-bleed rules
- business thesis for why Control exists

PR A is docs-first and intentionally avoids implementation.

## PR B/C: Control Admission And Disposition Governance Boundary

Status: current slice.

PR B/C replaces the previously separate admission and governance slices with one merged constitutional tranche. It defines:

- Control-side admission and intake boundary doctrine
- provenance, replay, lineage, and validation expectations for admitted compute artifacts
- rejection rules for malformed, stale, incomplete, or semantically inadmissible compute outputs
- Control-owned trust classes and disposition outcomes
- promotion, fallback, refusal, suppression, degrade, escalation, and hold-for-review governance
- comparison-aware governance without moving comparison computation into Control
- repair acceptance governance without moving repair math into Control
- the rule that admitted evidence may influence workflow state only through explicit Control-owned disposition

PR B/C remains docs-first. It does not add a runtime, validator implementation, policy engine, scheduler, artifact store, orchestration service, or product workflow.

## Near-Term Direction

Future slices should proceed only after PR A and PR B/C land the ownership, admission, and governance model.

Likely next areas:

- concrete artifact intake envelopes and eligibility records
- validation/admission record implementation
- explanation and surfacing posture
- workflow audit records
- policy hooks that remain Control-owned and compute-informed

These should be added in small slices. Each slice must preserve the split between compute evidence, workflow truth, and product action.

## Later Work

Later work may include orchestration, scheduling, runtime services, product integrations, and operational hardening. Those are not PR A or PR B/C.

The roadmap should not pull compute-kernel logic into Control or business workflow state into Compute.

## Non-Negotiable Direction

Ricercar-Control must become the layer where evidence-accountable structure is turned into accountable workflow state. It should not become a math engine, a UI app, or an unbounded automation platform.
