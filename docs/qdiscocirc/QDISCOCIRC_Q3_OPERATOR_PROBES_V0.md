# QDisCoCirc Q3 Operator Probes v0

Q3 finishes QDisCoCirc as a faithful operator/debug surface over already-owned Ricercar truth.

QDisCoCirc still does not own compute truth, workflow truth, compatibility judgment, release readiness, routing policy, rollback policy, or deployment behavior. It projects and links the typed records that Ricercar-Compute and Ricercar-Control already own.

## Canonical Operator Questions

The Q3 adapters expose stable question ids for:

- `what_happened`
- `why_held`
- `why_degraded`
- `why_fallback_only`
- `why_rollback_required`
- `what_changed`
- `what_blocked_promotion`
- `evidence_for_consequence`
- `ownership_split`

These are typed operator intents, not a natural-language parser and not a generic query engine.

## Canonical Corpus

The Q3 corpus pins these cross-repo scenarios:

- clean promotion
- review / hold
- compatibility block
- degraded operation
- fallback-only backend posture
- rollback-required incident posture

Each scenario preserves:

- Compute evidence refs as compute truth
- Control governance or routing refs as workflow consequence
- stable reason ids where the source record provides them
- a probe envelope id suitable for replay/debug

## Delta Semantics

Q3 delta summaries compare already-projected probe fields. They classify changes as:

- `compute_truth_changed`
- `control_consequence_changed`
- `mixed_ownership_changed`
- `probe_only_changed`
- `no_change`

This is not semantic diffing. It only reports whether existing probe source refs, summaries, or tags changed, and which ownership plane those changed refs came from.

## Finished Surface

QDisCoCirc is finished enough when an operator can ask what happened, why it happened, what changed, what blocked promotion, and which Compute evidence versus Control consequence produced the posture.

Future UI layers may render these probes, but they must not treat QDisCoCirc as a truth store or policy engine.
