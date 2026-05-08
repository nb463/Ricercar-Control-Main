# QDisCoCirc Probe Model v0

QDisCoCirc is a probe protocol with two ownership-specific adapters:

- Ricercar-Compute owns the compute adapter.
- Ricercar-Control owns the control adapter.
- The shared protocol is docs-defined here because Control is the cross-system explanation and governance plane.

QDisCoCirc is not the canonical truth store, compute semantic core, workflow state machine, scheduler, product UI, or substitute for typed contracts/evidence.

## Adapter Invariant

A QDisCoCirc adapter may only perform projection, normalization for probe display, graph/trace assembly, reference linking, and explanation formatting from already-owned truth.

A QDisCoCirc adapter may not perform semantic inference, workflow decisions, compatibility judgment, backend admissibility judgment, or canonical artifact generation.

## Minimal Object Model

`ProbeEnvelope` is the renderable/debuggable probe unit. It contains nodes, edges, traces, and one explanation payload.

`ProbeNode` represents an owned truth surface or probe-only structure. It must declare whether it is compute truth, control truth, or probe-only structure.

`ProbeEdge` links nodes with display relationships such as derived-from, explains, blocks, changes, or drills-down-to.

`ProbeTrace` gives an ordered walkthrough over node ids.

`ProbeExplanation` identifies the operator/debug walkthrough class:

- `ShowMeWhy`
- `ShowMeWhatChanged`
- `ShowMeWhatBlockedPromotion`

`SourceRef` links a node back to its owned source plane, source kind, source id, optional content hash, optional replay ref, and lineage refs.

## Q2 Walkthroughs

Q2 supports these first faithful probes:

- show why a workflow item was held, escalated, refused, or promoted;
- show what changed between two compatible probe envelopes;
- show what blocked promotion, especially across compatibility, readiness, backend, layout, and CUDA promotion evidence.

No Q2 walkthrough may invent new semantic status or workflow posture. It may only surface fields already present on the owned Compute or Control records.

## Q3 Operator Surface

Q3 finishes the protocol with stable operator question ids, canonical scenario views, and ownership-aware delta summaries. The canonical questions cover what happened, why a posture was held/degraded/fallback-only/rollback-required, what changed, what blocked promotion, which evidence justified a consequence, and which refs are Compute truth versus Control consequence.

Q3 remains projection-only. It does not add a query language, policy engine, truth store, release engine, or orchestration runtime.
