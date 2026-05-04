# Ricercar-Control Anti-Goals

These are explicit non-goals for Ricercar-Control, especially during PR A.

## Do Not Become Ricercar-Compute

Control must not reimplement:

- witnessed interaction kernels
- completion or cocompletion engines
- residual or repair math
- comparison kernels
- compute artifact derivation
- stable-structure extraction
- task-result classification owned by Compute

Control consumes compute artifacts. It does not become the compute plane.

## Do Not Become A Product App

Control must not own:

- product-specific UX
- customer-facing dashboards
- domain-specific forms and commands
- business-system integrations that belong in app layers
- concrete business action surfaces

Products may use Control. They should not be collapsed into it.

## Do Not Let Artifacts Become Workflow Truth Automatically

Compute artifacts may inform decisions. They must not silently create:

- approvals
- assignments
- suppression decisions
- escalation decisions
- pursuit decisions
- final human audit state

Every workflow consequence must be explicit and auditable.

## Do Not Add Runtime Before Doctrine

PR A must not add:

- runtime implementation
- orchestration code
- intake validators
- policy engines
- schedulers
- distributed systems
- envelope implementation
- compute-kernel logic

Those may become appropriate after the constitutional boundary is clear.

## Do Not Hide Ambiguity

Control exists because ambiguity matters. It should not flatten disagreement, uncertainty, local/global view differences, approximation, or degraded evidence into a fake certainty just because a workflow wants a simple answer.

The system should preserve enough lineage and explanation to help humans understand why a decision state exists.
