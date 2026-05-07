# Control Orchestration And Routing Runtime v0

PR E adds the first narrow runnable Control runtime slice for routing admitted compute evidence into accountable workflow consequence.

It is not a distributed scheduler, product UI, deployment system, release governance layer, or compute kernel.

## Ownership

Ricercar-Compute owns:

- semantic truth
- backend admissibility evidence
- plugin compatibility evidence
- cache posture evidence
- compatibility gate evidence
- release readiness evidence

Ricercar-Control owns:

- queueing and scheduling posture
- routing intent
- orchestration state
- execution command
- workflow consequence
- operator-facing explanation
- audit records

Control may consume Compute evidence. It must not regenerate Compute truth locally.

## Control-Side Ontology Map

PR E keeps these layers distinct:

| Layer | Owner | PR E role |
| --- | --- | --- |
| Compute evidence | Compute | Typed input summarized by PR D evidence shapes |
| Admitted evidence | Control | Evidence that passed Control intake and may enter governance |
| Workflow state | Control | Routing/orchestration posture created by Control |
| Operator-facing explanation | Control | Structured explanation payload for consequential acts |
| Execution command / routing consequence | Control | Explicit command such as promote, hold, escalate, refuse, recompute, fallback, or suppress |

The movement from compute evidence to workflow consequence must be explicit, typed, auditable, and explainable.

## Runtime Surface

PR E introduces:

- `QueueableWorkItem`
- `RoutingIntentKind`
- `RoutingDecision`
- `ExecutionCommand`
- `OrchestrationState`
- `OrchestrationAuditRecord`
- `RoutingExplanationPayload`

The runtime is local and deterministic. It does not persist records, distribute work, or schedule across services.

## Evidence Consumption

Routing consumes typed summaries already established in PR D and finalized by Compute PR34-PR36:

- `CachePolicySummary`
- `CompatibilityGateSummary`
- `ReleaseReadinessSummary`
- `BackendAdmissibility`
- `BackendRuntimePostureSummary`
- plugin compatibility summaries

Routing records typed Control consequences and preserves Compute-owned reason ids in the routing explanation payload.

## Consequential Acts

Every consequential routing act emits:

- routing decision
- execution command
- orchestration state
- PR D explanation bundle
- routing explanation payload
- audit key

Consequential acts include:

- promote for execution
- hold for review
- escalate
- refuse execution
- trigger recompute
- route to fallback
- suppress ordinary routing

## Recompute Triggers

Control may trigger recompute when Compute cache posture says recompute is required or reuse is refused.

Control does not recompute cache truth. It records the workflow consequence and preserves the Compute recompute reason id.

## Compatibility And Release Readiness

Compatibility gate blocking remains routing-relevant. Release readiness blocking remains routing-relevant.

Control must not turn a blocked compatibility or readiness posture into ordinary execution. Those postures route to escalation or review consequences.

## Backend / CUDA / Accelerated Routing

Accelerated or CUDA-leaning routes fail closed unless typed evidence exists.

At minimum:

- accelerated routing is not eligible without backend admissibility evidence
- accelerated routing is not eligible without release readiness evidence
- accelerated routing is not eligible without clean compatibility-gate evidence
- backend runtime parity, layout, precision, or canonicalization review prevents ordinary accelerated execution
- missing or unknown evidence cannot silently become eligibility

PR E does not finish PR37 backend/layout governance. It leaves a typed seam for richer backend/layout policy.

## Non-Scope

PR E does not add:

- Compute semantic validation
- cache recomputation
- plugin loading
- deployment or release governance
- distributed scheduling
- routing services
- product UI
- QDisCoCirc Q2 probe mappings
