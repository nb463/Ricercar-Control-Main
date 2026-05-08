# QDisCoCirc Identity And Reference Rules v0

Probe identity is stable enough for debugging and replay-linking, but it is not canonical artifact identity.

## Source Planes

`compute` refers to Ricercar-Compute-owned evidence and artifacts.

`control` refers to Ricercar-Control-owned workflow, admission, governance, explanation, routing, and audit truth.

`probe` refers to QDisCoCirc-only structure such as diff nodes or trace assembly nodes.

## Reference Shape

Every probe node should preserve:

- source plane;
- source kind;
- source id;
- content hash where the source has one;
- replay reference where the source has one;
- lineage references where the source exposes them;
- whether the source is a helper/projection surface rather than public canonical truth.

## Linking Rules

Compute adapter references point back to compute artifact keys, helper evidence keys, content hashes, replay notes, stable object keys, witness keys, or lineage references.

Control adapter references point back to admission records, governance records, explanation bundles, routing decisions, execution commands, audit records, and the compute evidence keys consumed by those records.

Backend-local CUDA layout references must remain backend-local shadow evidence. They must not be represented as canonical public artifact truth unless Compute has already rematerialized the result through the public artifact boundary.
