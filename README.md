# Ricercar-Control

Ricercar-Control is the decision plane, Core OS, and workflow-truth layer for Ricercar systems.

It is the better half of Ricercar-Compute, but it is not a second compute kernel. Ricercar-Compute owns derived compute truth: witnessed interactions, lawful derived artifacts, stable structure, observables, residuals, repair evidence, and replayable semantic lineage. Ricercar-Control owns the operational truth that decides what those artifacts mean for people, products, and workflows.

The boundary is deliberate:

- Compute artifacts may inform workflow decisions.
- Compute artifacts must not silently become workflow truth.
- Control may govern intake, trust, disposition, explanation, orchestration, and workflow consequences.
- Control must not absorb kernel mathematics or reimplement compute semantics.
- Product and app layers own concrete UX and business action surfaces.

Start here:

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/ROADMAP.md](docs/ROADMAP.md)
- [docs/ANTI_GOALS.md](docs/ANTI_GOALS.md)
- [docs/GLOSSARY.md](docs/GLOSSARY.md)
- [docs/CONTROL_ADMISSION_AND_DISPOSITION_BOUNDARY_V0.md](docs/CONTROL_ADMISSION_AND_DISPOSITION_BOUNDARY_V0.md)
- [docs/CONTROL_EXPLANATION_AND_SURFACING_GRAMMAR_V0.md](docs/CONTROL_EXPLANATION_AND_SURFACING_GRAMMAR_V0.md)
- [docs/qdiscocirc/QDISCOCIRC_PROBE_MODEL_V0.md](docs/qdiscocirc/QDISCOCIRC_PROBE_MODEL_V0.md)

## Business Thesis

Ricercar exists to help humans see important structure earlier, more honestly, and with a clearer path back to evidence.

The system is for products where meaningful, replayable, evidence-accountable structure must appear early enough for humans to act before an opportunity disappears or a mistake becomes expensive. These are workflows where evidence is incomplete, disagreement matters, timing matters, local and global views can diverge, approximation is unavoidable, and trust is expensive to lose.

Ricercar helps humans make higher-value decisions under ambiguity by surfacing earlier, more stable, more evidence-accountable structure than simpler systems can reliably provide.

That is why Ricercar's mathematical commitments matter:

- interaction-first semantics
- lawful modulation and reduction
- stable-structure extraction
- principled observables
- accountable factorization and repair

The business aim is to become a shared derived-structural-intelligence core for tools where ambiguity is expensive, timing matters, and humans still need to trust what they are seeing.

## Stack Principle

Across ACE, MSURV, and future Ricercar applications:

- app layers prepare and route relational material
- Ricercar-Compute extracts relevant structural information
- Ricercar-Control decides how that information enters workflow truth
- humans review for relevant purposes
- the system learns over time which structures, modulations, reductions, explanations, and repair signals are most decision-useful

This stack keeps each layer honest. Compute produces evidence-accountable derived structure. Control governs workflow consequences. Products shape the user-facing business action.

## Boundary Ownership

Ricercar-Compute may produce:

- evidence bundles
- stable cores
- instability surfaces
- next-best probes
- task-specific artifacts
- hypotheses
- scores
- contrasts

Ricercar-Control owns:

- artifact intake posture
- trust and disposition state
- explanation and surfacing policy
- assignments
- approvals
- suppression and escalation choices
- pursuit decisions
- the final audit trail of human workflow action

Product and app layers own:

- concrete business operations
- user experience
- workflow-specific commands and forms
- integrations with customer-facing systems
- business-specific labels, dashboards, and action surfaces

This boundary is not a weakness. It is a trust-preserving feature.

## Current State And Sequencing

The canonical cross-repo sequencing now lives in `RICERCAR_INTEGRATED_ARCHITECTURE_AND_FINISH_PLAN.md` at the workspace root. This repo's docs describe Control's owned boundaries and local validation path; the integrated plan is the source of truth for what comes next across Compute, Control, and QDisCoCirc.

PR A through PR F are the landed Control constitution and operational substrate:

- PR A and PR B/C define ownership, admission, and disposition doctrine.
- PR D implements admission, governance, explanation, and surfacing grammar.
- PR E implements the narrow routing/orchestration runtime and PR37-aware accelerated routing consumption.
- PR F implements Control release-readiness, operational governance, transition guards, incident posture, and rollback/degrade/hold discipline.

With that constitutional tail in place, Control should stay in support mode while the center of gravity moves to detector proof and kernel reality. Future Control changes should be small, evidence-consuming slices driven by real detector outputs, not new Control-side semantic invention.

## Documents

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Anti-goals](docs/ANTI_GOALS.md)
- [Glossary](docs/GLOSSARY.md)
- [Control Admission And Disposition Boundary v0](docs/CONTROL_ADMISSION_AND_DISPOSITION_BOUNDARY_V0.md)
- [Control Explanation And Surfacing Grammar v0](docs/CONTROL_EXPLANATION_AND_SURFACING_GRAMMAR_V0.md)
- [Control Orchestration And Routing Runtime v0](docs/CONTROL_ORCHESTRATION_AND_ROUTING_RUNTIME_V0.md)
- [Control Operational Hardening And Release Governance v0](docs/CONTROL_OPERATIONAL_HARDENING_AND_RELEASE_GOVERNANCE_V0.md)
- [QDisCoCirc Probe Model v0](docs/qdiscocirc/QDISCOCIRC_PROBE_MODEL_V0.md)

## Local Validation

GitHub CI runs a small trust lane for formatting plus PR D/PR E smoke tests, and a proof lane for the full test suite plus clippy. The same gates can be run locally:

- `cargo fmt --check`
- `cargo test -q`
- `cargo clippy --all-targets -- -D warnings -A clippy::too_many_arguments`
