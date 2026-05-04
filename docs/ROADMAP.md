# Ricercar-Control Roadmap

Ricercar-Control starts after the current Ricercar-Compute artifact-boundary and local-cache/recompute tranches, with constitutional boundaries before runtime machinery.

## PR A: Constitutional Skeleton And Boundary Ownership

Status: current slice.

PR A establishes:

- Ricercar-Control as decision-core and workflow-truth owner
- Ricercar-Compute as math engine, compute plane, and kernel family
- product/app layers as UX and concrete business action owners
- artifact intake doctrine
- workflow truth doctrine
- truth non-bleed rules
- business thesis for why Control exists

PR A is docs-first and intentionally avoids implementation.

## Near-Term Direction

Future slices should proceed only after PR A lands the ownership model.

Likely next areas:

- artifact intake envelopes and eligibility state
- compute artifact validation/admission records
- trust and disposition vocabulary
- explanation and surfacing posture
- workflow audit records
- policy hooks that remain Control-owned and compute-informed

These should be added in small slices. Each slice must preserve the split between compute evidence, workflow truth, and product action.

## Later Work

Later work may include orchestration, scheduling, runtime services, product integrations, and operational hardening. Those are not PR A.

The roadmap should not pull compute-kernel logic into Control or business workflow state into Compute.

## Non-Negotiable Direction

Ricercar-Control must become the layer where evidence-accountable structure is turned into accountable workflow state. It should not become a math engine, a UI app, or an unbounded automation platform.
