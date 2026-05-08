//! Control-owned QDisCoCirc probe adapter.
//!
//! This crate projects existing Ricercar-Control truth into probe envelopes for
//! operator/debug walkthroughs. It may assemble traces, normalize labels, link
//! references, and format explanation material from Control-owned records. It
//! must not perform semantic inference, workflow decisions, compatibility
//! judgment, backend admissibility judgment, routing policy, or canonical
//! artifact generation.

pub mod envelope;
pub mod mappers;
pub mod model;
pub mod refs;
pub mod render_support;
pub mod traces;

pub use envelope::{probe_envelope, ProbeEnvelopeBuilder};
pub use model::{
    ProbeChange, ProbeEdge, ProbeEdgeKind, ProbeExplanation, ProbeNode, ProbeNodeRole, ProbeTrace,
    ProbeWalkthroughKind, SourcePlane,
};
pub use refs::SourceRef;
