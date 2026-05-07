# Control Explanation And Surfacing Grammar v0

PR D adds the first narrow runnable Control slice. It turns compute evidence summaries into admission records, Control-owned trust and disposition, and workflow-auditable explanation bundles.

This is not a scheduler, orchestrator, product UI, deployment policy, or compute kernel. It is the smallest typed layer that lets Control say what entered workflow space, what did not, what disposition was assigned, and why.

## Boundary Position

Ricercar-Compute remains the source of compute truth. It produces typed evidence such as plugin compatibility, cache posture, compatibility-gate verdicts, release/readiness posture, backend admissibility, and backend runtime or memory-layout posture.

Ricercar-Control consumes those summaries. It does not recompute plugin compatibility, cache freshness, boundary compatibility, release readiness, backend parity, or CUDA memory-layout semantics.

Products consume Control posture and explanation. They still own concrete screens, workflow-specific labels, and business actions.

## Admission Envelope

An admission envelope carries:

- evidence key
- compute evidence kind
- provenance, including source system, workflow context, artifact key, content hash, replay reference, and lineage
- compute validation posture
- compute semantic status
- typed compute evidence summary

Admission fails closed when identity, provenance, replay reference, lineage, content hash, validation posture, or semantic admissibility is insufficient.

Admission is not approval. It only decides whether evidence is eligible for Control governance.

## Governance Record

Governance assigns Control-owned workflow meaning to admitted evidence:

- trust class
- disposition
- explicit governance reasons

Trust and disposition do not rewrite compute truth. They record Control interpretation for workflow consequence.

## Explanation Bundle

An explanation bundle assembles a control trace into:

- admission records
- governance records
- aggregate trust class
- aggregate disposition
- incident/debug fragments
- operator, downstream-system, and audit surfacing directives
- audit summary
- optional diagram hint

Bundles are workflow-auditable. They preserve why evidence was admitted, refused, degraded, escalated, or held for review.

## Surfacing Grammar

The surfacing grammar is typed and audience-aware:

- operator directives explain the Control posture humans should see
- downstream-system directives expose machine-readable workflow posture
- audit directives preserve reviewable explanation state

Surfacing directives are not product UI and not routing commands. They are Control-owned explanation posture that products and later runtime slices may consume.

## Evidence Chain

PR D keeps the evidence chain explicit:

1. backend capability or backend admissibility
2. plugin compatibility
3. cache or recompute posture
4. compatibility-gate verdict
5. release or boundary readiness posture

Control can explain how this chain affects workflow posture without inferring compute truth from logs or prose.

## CUDA Transition Posture

Backend runtime and memory-layout evidence are consumed as typed posture:

- backend role
- layout version
- memory-layout posture
- precision posture
- canonicalization posture
- CPU/reference parity oracle

CPU/reference semantics remain the canonical oracle. Backend-local device layout, transfer, and packing details remain implementation evidence, not public compute artifact truth.

## Diagram Hints

PR D includes only a lightweight explanation hint vocabulary:

- wires are spaces of variables or workflow/evidence channels
- boxes are typed Control interpretation steps
- traces represent forgotten or unadmitted evidence
- contractions represent composition of evidence into an explanation bundle
- splits distinguish compute truth from workflow truth

Diagram hints are explanatory. They do not add compute semantics, product UX, or runtime behavior.

## Non-Scope

PR D does not implement:

- orchestration or scheduling
- product/app screens
- deployment or rollout policy
- org-wide governance
- compute kernels or recomputation
- hidden promotion, fallback, or suppression policy
