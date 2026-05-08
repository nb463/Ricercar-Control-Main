# QDisCoCirc Control Mapping Rules v0

The Control adapter projects Ricercar-Control truth. It does not recompute Compute truth and does not make new workflow decisions.

## Q2 Mapped Surfaces

Q2 maps:

- `AdmissionEnvelope`;
- `AdmissionRecord`;
- `GovernanceRecord`;
- trust and disposition outcomes;
- `ExplanationBundle`;
- explanation fragments, diagram evidence flows, posture flows, and surfacing directives;
- orchestration routing decisions, execution commands, routing explanation payloads, and audit records already present in Control.

## Drill-Down Rule

A routing or workflow consequence probe should drill down in this order:

1. workflow consequence or execution command;
2. routing decision or governance record;
3. explanation bundle / routing explanation payload;
4. admitted or rejected compute evidence keys;
5. compute-owned reason ids, replay references, and lineage references where present.

Control probe output may say that Control held, escalated, refused, suppressed, degraded, or promoted workflow consequence. It may not re-label compute compatibility, readiness, backend admissibility, cache posture, or CUDA parity truth.
