mod commands;
mod error;
mod events;
mod queries;
mod types;

use personaflux_core::Simulation;

pub use commands::{
    pf_api_version, pf_faction_add, pf_member_add, pf_model_version,
    pf_relationship_faction_to_faction_clear, pf_relationship_faction_to_faction_set,
    pf_relationship_faction_to_member_clear, pf_relationship_faction_to_member_set,
    pf_relationship_member_to_member_clear, pf_relationship_member_to_member_set,
    pf_simulation_advance_to, pf_simulation_create, pf_simulation_destroy,
    pf_simulation_random_seed, pf_simulation_step, pf_simulation_submit_direct_witness,
    pf_simulation_submit_direct_witness_batch,
};
pub use error::{
    PF_BUFFER_TOO_SMALL, PF_INTERNAL_ERROR, PF_INVALID_ARGUMENT, PF_INVALID_STATE, PF_NOT_FOUND,
    PF_OK, PF_SERIALIZATION_ERROR, PF_VERSION_MISMATCH, PfResult,
};
pub use events::{pf_events_clear, pf_events_count, pf_events_read, pf_last_error_message_copy};
pub use queries::{
    pf_faction_name_get, pf_member_affinity_get, pf_member_state_get, pf_memories_count,
    pf_memories_read, pf_memory_get, pf_relationship_effective_member_get,
    pf_relationship_faction_to_faction_get, pf_relationship_faction_to_member_get,
    pf_relationship_member_to_member_get, pf_simulation_current_tick,
};
pub use types::{
    PF_ABI_VERSION, PF_EVENT_AFFINITY_CHANGED, PF_EVENT_DEED_EVALUATED, PF_EVENT_FACTION_ADDED,
    PF_EVENT_MEMBER_ADDED, PF_EVENT_MEMORY_EXPIRED, PF_EVENT_MEMORY_REMEMBERED,
    PF_EVENT_MEMORY_UPGRADED, PF_EVENT_PAD_CHANGED, PF_EVENT_RELATIONSHIP_CHANGED,
    PF_EVENT_TIME_ADVANCED, PF_MEMORY_DECISION_LONG_TERM, PF_MEMORY_DECISION_SHORT_TERM,
    PF_MEMORY_LONG_TERM, PF_MEMORY_NONE, PF_MEMORY_SHORT_TERM, PF_MODEL_VERSION,
    PF_RELATIONSHIP_FACTION_TO_FACTION, PF_RELATIONSHIP_FACTION_TO_MEMBER,
    PF_RELATIONSHIP_MEMBER_TO_MEMBER, PF_SUBMISSION_APPLIED, PF_SUBMISSION_DUPLICATE,
    PfDirectWitnessDeed, PfDirectWitnessOutcome, PfEvaluationResult, PfEvent, PfMemberState,
    PfMemoryRecord, PfPad, PfRelationshipLookup, PfSubmissionResult,
};

/// Opaque C handle for one isolated simulation.
#[repr(C)]
pub struct PfSimulation {
    pub(crate) inner: Simulation,
}
