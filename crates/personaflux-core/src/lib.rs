#![forbid(unsafe_code)]

mod bounds;
mod evaluation;
mod pad;
mod relationship;
mod simulation;
mod values;

pub use bounds::{apply_affinity_delta, apply_bounded_delta, apply_pad_value_delta};
pub use evaluation::{
    DirectWitnessInput, EvaluationPolicyV1, EvaluationResult, MemoryClassification, MemoryDecision,
    evaluate_direct_witness,
};
pub use pad::Pad;
pub use relationship::{RelationshipLayer, RelationshipLookup, RelationshipSubject};
pub use simulation::{Error, FactionId, MemberId, Simulation, SimulationConfig, SimulationEvent};
pub use values::{Affinity, Aggression, Confidence, Impact, PadValue, ValueError};
