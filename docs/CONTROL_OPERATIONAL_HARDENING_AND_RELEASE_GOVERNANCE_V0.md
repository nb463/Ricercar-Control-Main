# Control Operational Hardening And Release Governance v0

PR F makes Ricercar-Control operable as the decision plane. It adds typed Control-owned readiness and governance posture without adding deployment automation, product UX, or Compute semantics.

## Ownership Boundary

Compute proves:

- compatibility-gate posture
- release-readiness posture
- backend admissibility
- CUDA layout, parity, canonicalization, and promotion evidence
- cache/recompute posture

Control governs:

- whether the decision plane itself is releasable
- policy-version compatibility
- operational hold, degrade, escalation, fallback, and rollback posture
- audit completeness for workflow-truth transitions
- whether a workflow consequence may move from held/degraded/fallback/rollback posture back to promotion

Compute readiness evidence may inform Control release governance. It does not become Control release governance by itself.

## Control Release Readiness

The Control-owned readiness report checks only decision-plane obligations:

- policy set is present and versioned
- policy compatibility is compatible, review-required, or breaking
- audit requirements are satisfied
- governance trace corpus is present and green
- workflow transition guards are present
- required upstream Compute evidence surfaces are consumable
- rollback doctrine is present

Statuses are:

- `Ready`: Control can be considered a credible decision-plane release candidate.
- `NeedsReview`: Control is not blocked, but explicit review is required before blessing.
- `Blocked`: Control is not releasable as decision-plane evidence.

## System Release Governance Posture

The system release governance record combines:

- Control release readiness
- Compute compatibility-gate evidence
- Compute release-readiness evidence
- PR37 CUDA/backend promotion evidence
- orchestration audit completeness
- optional Control-owned incident posture

Postures are:

- `Promotable`
- `HoldForReview`
- `DegradedButGovernable`
- `FallbackOnly`
- `RollbackRequired`
- `Blocked`

This posture is workflow/governance truth. It does not recompute Compute compatibility, CUDA parity, cache freshness, or release readiness.

## Incident And Rollback Governance

Incidents are typed Control-owned workflow facts. Supported responses are:

- operational hold
- degraded operation
- escalation required
- rollback required
- rollback in effect
- blocked

Rollback is a Control workflow consequence. Compute may supply the evidence that makes rollback necessary, but Control owns the rollback state and audit record.

## Policy Version Compatibility

Governance policy evolution is compatibility-sensitive:

- `Compatible`: no release hold from policy movement.
- `ReviewRequired`: release readiness becomes review-required.
- `Breaking`: release readiness is blocked until the policy movement is explicitly handled.

Policy compatibility is Control-owned. It does not change compute artifact truth.

## Transition Guards

Consequential transitions must remain auditable:

- missing audit records block transition
- missing explanation payloads block transition
- missing evidence references block transition
- moving from hold, degrade, fallback, rollback, or blocked posture back to promotion requires explicit review

No Control release governance path may silently promote through missing evidence or incomplete audit.

## Scenario Corpus

The PR F test corpus pins:

- promotion
- hold for review
- rollback required
- incompatibility blocked
- degraded but governable operation
- escalation required

These are typed test scenarios, not product workflows and not deployment automation.
