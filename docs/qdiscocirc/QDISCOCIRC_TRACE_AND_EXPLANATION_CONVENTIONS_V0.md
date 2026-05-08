# QDisCoCirc Trace And Explanation Conventions v0

QDisCoCirc traces are lightweight operator/debug paths. They are not workflow state and not compute artifact lineage.

## Trace Conventions

Trace titles should be short and should name the walkthrough, such as "CUDA promotion drill-down" or "Workflow consequence drill-down".

Trace node order should follow ownership:

- Compute evidence before Control interpretation when explaining why;
- before envelope before after envelope when showing what changed;
- backend admissibility / layout / parity evidence before promotion posture when showing what blocked promotion.

## Explanation Conventions

Probe explanations should preserve typed reason ids from the source record. They may format them for readability, but they must not collapse them into looser prose when the exact id is available.

`ShowMeWhy` explains the source record chain behind a consequence.

`ShowMeWhatChanged` compares probe node summaries and tags only. It does not perform ad hoc semantic diffing.

`ShowMeWhatBlockedPromotion` surfaces blocking reason ids from compatibility, readiness, backend runtime, layout, parity, cache, or routing evidence without inventing a new policy.

## Q1/Q2 Exemplars

Witnessed interaction to compute probe:

`witnessed interaction -> lawful cocompletion/completion -> residual/repair evidence -> compute probe envelope`

Release readiness to Control explanation:

`release readiness report -> admission record -> governance disposition -> explanation bundle -> control probe envelope`

Backend/layout posture to routing review:

`backend admissibility -> CUDA layout manifest -> CUDA promotion evidence -> Control release/backend posture -> routing hold/escalation probe trace`
