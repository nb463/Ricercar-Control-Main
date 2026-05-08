# QDisCoCirc Compute Mapping Rules v0

The Compute adapter projects Ricercar-Compute truth. It does not generate truth.

## Q2 Mapped Surfaces

Q2 maps:

- `ComputeArtifactV1` envelopes;
- witnessed interaction helper surfaces;
- lawful cocompletion outputs;
- lawful completion outputs;
- residual traces;
- repair approximation reports;
- cache-policy decisions;
- contract compatibility reports;
- release-readiness reports;
- CUDA backend promotion evidence, including backend admissibility and layout manifest references.

## Mapping Discipline

Public compute artifacts should be marked as compute truth.

Helper/projection surfaces should be marked as helper surfaces. They are probeable and evidence-bearing, but they do not expand the public artifact matrix.

CUDA layout manifests and backend-local runtime posture should be shown as backend-local shadow evidence. Probe output may show layout version, transfer semantics, precision, packing, canonicalization, parity, and promotion posture, but it must not treat backend-local layout as canonical semantic artifact truth.

The adapter may preserve typed reasons such as parity-over-budget, layout-review-required, compatibility drift, release-readiness blocking, or cache blocked posture. It may not decide whether those reasons should cause workflow consequence.
