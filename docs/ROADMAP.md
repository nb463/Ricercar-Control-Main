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

Status: landed.

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

## PR D: Explanation Assembly And Surfacing Grammar v0

Status: current slice.

PR D adds the first narrow runnable Control implementation:

- Control-side admission envelopes and admission records
- admission outcomes and rejection reasons
- trust class assignment and disposition semantics for admitted evidence
- compute-evidence summary adapters for plugin compatibility, cache policy, compatibility gates, release readiness, backend admissibility, and backend runtime posture
- trace-to-explanation assembly
- workflow-auditable explanation bundles
- operator, downstream-system, and audit surfacing directives
- lightweight diagram hints for explanation surfaces only

PR D does not add scheduling, orchestration, product UX, deployment policy, org-wide governance, compute kernels, or recomputation logic.

## PR E: Orchestration, Scheduling, And Routing Runtime v0

Status: current slice.

PR E adds the first narrow operational Control runtime:

- queueable work items and routing intents
- routing decisions and execution commands
- orchestration state and audit records
- structured routing explanation payloads for every consequential act
- typed consumption of Compute cache posture, compatibility-gate posture, release readiness, PR37 CUDA/backend promotion posture, backend admissibility, backend runtime posture, and plugin compatibility evidence
- fail-closed accelerated-routing doctrine when PR37 CUDA/backend promotion evidence, release readiness, or compatibility evidence is missing or blocked

PR E does not add distributed scheduling, product UX, deployment policy, release governance, compute kernels, recomputation logic, or full PR37 backend/layout governance.

## PR F: Operational Hardening And Release Governance v0

Status: current slice.

PR F adds the first typed operational hardening layer for the Control decision plane:

- Control release-readiness reports for policy presence, audit requirements, governance trace corpus, transition guards, upstream Compute evidence consumption, and rollback doctrine
- policy-version compatibility review with compatible, review-required, and breaking postures
- system release-governance posture over Compute compatibility, Compute release readiness, PR37 CUDA/backend promotion evidence, Control readiness, orchestration audit completeness, and Control-owned incident posture
- typed incident responses for hold, degrade, escalation, rollback required, rollback in effect, and blocked operation
- workflow-truth transition guards that prevent silent promotion through missing audit, missing explanations, missing evidence, or unreviewed recovery from hold/degrade/fallback/rollback
- a governance-state trace corpus covering promotion, hold, escalation, rollback, incompatibility, and degraded operation

PR F does not add deployment automation, cloud rollout tooling, product UX, Compute truth recomputation, or org-wide release management.

## Near-Term Direction

Future slices should proceed only after PR A, PR B/C, PR D, PR E, and PR F preserve the ownership, admission, governance, explanation, routing, and operational hardening model.

Likely next areas:

- workflow audit records
- policy hooks that remain Control-owned and compute-informed

These should be added in small slices. Each slice must preserve the split between compute evidence, workflow truth, and product action.

## Later Work

Later work may include broader runtime services, product integrations, and deployment-specific operations. Those are not PR A through PR F.

The roadmap should not pull compute-kernel logic into Control or business workflow state into Compute.

## Non-Negotiable Direction

Ricercar-Control must become the layer where evidence-accountable structure is turned into accountable workflow state. It should not become a math engine, a UI app, or an unbounded automation platform.
