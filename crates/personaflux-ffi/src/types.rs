use personaflux_core::{
    Affinity, Aggression, DirectWitnessDeed, DirectWitnessOutcome, DirectWitnessSubmission,
    EvaluationResult, FactionId, Impact, MemberId, MemoryKind, MemoryRecord, Pad,
    RelationshipLayer, RelationshipLookup, SimulationEvent,
};

use crate::error::{PfResult, invalid_argument, map_core_error, version_mismatch};

pub const PF_ABI_VERSION: u32 = 0;
pub const PF_MODEL_VERSION: u32 = 1;

pub const PF_SUBMISSION_APPLIED: u32 = 1;
pub const PF_SUBMISSION_DUPLICATE: u32 = 2;

pub const PF_RELATIONSHIP_MEMBER_TO_MEMBER: u32 = 1;
pub const PF_RELATIONSHIP_FACTION_TO_MEMBER: u32 = 2;
pub const PF_RELATIONSHIP_FACTION_TO_FACTION: u32 = 3;

pub const PF_MEMORY_SHORT_TERM: u32 = 1;
pub const PF_MEMORY_LONG_TERM: u32 = 2;

pub const PF_MEMORY_NONE: u32 = 0;
pub const PF_MEMORY_DECISION_SHORT_TERM: u32 = 1;
pub const PF_MEMORY_DECISION_LONG_TERM: u32 = 2;

pub const PF_EVENT_FACTION_ADDED: u32 = 1;
pub const PF_EVENT_MEMBER_ADDED: u32 = 2;
pub const PF_EVENT_RELATIONSHIP_CHANGED: u32 = 3;
pub const PF_EVENT_DEED_EVALUATED: u32 = 4;
pub const PF_EVENT_AFFINITY_CHANGED: u32 = 5;
pub const PF_EVENT_PAD_CHANGED: u32 = 6;
pub const PF_EVENT_MEMORY_REMEMBERED: u32 = 7;
pub const PF_EVENT_MEMORY_UPGRADED: u32 = 8;
pub const PF_EVENT_MEMORY_EXPIRED: u32 = 9;
pub const PF_EVENT_TIME_ADVANCED: u32 = 10;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfPad {
    pub pleasure: f32,
    pub arousal: f32,
    pub dominance: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfRelationshipLookup {
    pub struct_size: u32,
    pub api_version: u32,
    pub present: u8,
    pub reserved: [u8; 3],
    pub source: u32,
    pub affinity: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfEvaluationResult {
    pub concern: f32,
    pub effective_confidence: f32,
    pub raw_affinity_delta: f32,
    pub raw_pad_delta: PfPad,
    pub event_intensity: f32,
    pub memory_salience: f32,
    pub memory_kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfDirectWitnessDeed {
    pub struct_size: u32,
    pub api_version: u32,
    pub deed_id: u64,
    pub observer: u64,
    pub actor: u64,
    pub target: u64,
    pub impact: f32,
    pub aggression: f32,
    pub has_target: u8,
    pub threatens_observer: u8,
    pub reserved: [u8; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfMemoryRecord {
    pub struct_size: u32,
    pub api_version: u32,
    pub observer: u64,
    pub deed_id: u64,
    pub actor: u64,
    pub target: u64,
    pub has_target: u8,
    pub reserved: [u8; 3],
    pub impact: f32,
    pub aggression: f32,
    pub salience: f32,
    pub kind: u32,
    pub created_tick: u64,
    pub expires_at: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfDirectWitnessOutcome {
    pub struct_size: u32,
    pub api_version: u32,
    pub deed_id: u64,
    pub observer: u64,
    pub actor: u64,
    pub target: u64,
    pub has_target: u8,
    pub reserved_target: [u8; 3],
    pub relationship: PfRelationshipLookup,
    pub evaluation: PfEvaluationResult,
    pub previous_affinity: f32,
    pub current_affinity: f32,
    pub previous_pad: PfPad,
    pub current_pad: PfPad,
    pub has_memory: u8,
    pub reserved_memory: [u8; 3],
    pub memory: PfMemoryRecord,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfSubmissionResult {
    pub struct_size: u32,
    pub api_version: u32,
    pub kind: u32,
    pub reserved: u32,
    pub observer: u64,
    pub deed_id: u64,
    pub outcome: PfDirectWitnessOutcome,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfMemberState {
    pub struct_size: u32,
    pub api_version: u32,
    pub faction_id: u64,
    pub pad: PfPad,
}

/// A tagged, fixed-size event record. Fields not used by the tag are zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PfEvent {
    pub struct_size: u32,
    pub api_version: u32,
    pub kind: u32,
    pub reserved: u32,
    pub faction_id: u64,
    pub member_id: u64,
    pub deed_id: u64,
    pub observer: u64,
    pub actor: u64,
    pub target: u64,
    pub has_target: u8,
    pub reserved_target: [u8; 3],
    pub relationship_layer: u32,
    pub relationship_source_id: u64,
    pub relationship_target_id: u64,
    pub previous_present: u8,
    pub current_present: u8,
    pub reserved_relationship: [u8; 2],
    pub previous_affinity: f32,
    pub current_affinity: f32,
    pub evaluation: PfEvaluationResult,
    pub previous_pad: PfPad,
    pub current_pad: PfPad,
    pub memory_kind: u32,
    pub previous_memory_kind: u32,
    pub current_memory_kind: u32,
    pub salience: f32,
    pub previous_tick: u64,
    pub current_tick: u64,
}

pub fn expected_size<T>() -> u32 {
    std::mem::size_of::<T>() as u32
}

pub fn validate_input_header(
    struct_size: u32,
    api_version: u32,
    expected_size: u32,
) -> Result<(), PfResult> {
    if api_version != PF_ABI_VERSION {
        return Err(version_mismatch());
    }
    if struct_size < expected_size {
        return Err(invalid_argument("struct_size is smaller than the v0 type"));
    }
    Ok(())
}

pub fn validate_output_size(out_size: u32, expected_size: u32) -> Result<(), PfResult> {
    if out_size < expected_size {
        return Err(invalid_argument(
            "output buffer is smaller than the v0 type",
        ));
    }
    Ok(())
}

pub fn parse_deed(raw: &PfDirectWitnessDeed) -> Result<DirectWitnessDeed, PfResult> {
    validate_input_header(
        raw.struct_size,
        raw.api_version,
        expected_size::<PfDirectWitnessDeed>(),
    )?;
    if raw.has_target > 1 || raw.threatens_observer > 1 {
        return Err(invalid_argument("boolean fields must be 0 or 1"));
    }
    if raw.has_target == 0 && raw.target != 0 {
        return Err(invalid_argument(
            "target must be zero when has_target is zero",
        ));
    }
    let impact = Impact::new(raw.impact).map_err(|error| map_core_error(&error.into()))?;
    let aggression =
        Aggression::new(raw.aggression).map_err(|error| map_core_error(&error.into()))?;
    Ok(DirectWitnessDeed::new(
        raw.deed_id,
        MemberId::from_raw(raw.observer),
        MemberId::from_raw(raw.actor),
        (raw.has_target == 1).then(|| MemberId::from_raw(raw.target)),
        impact,
        aggression,
        raw.threatens_observer == 1,
    ))
}

pub fn affinity(value: f32) -> Result<Affinity, PfResult> {
    Affinity::new(value).map_err(|error| map_core_error(&error.into()))
}

pub fn faction_id(value: u64) -> FactionId {
    FactionId::from_raw(value)
}

pub fn member_id(value: u64) -> MemberId {
    MemberId::from_raw(value)
}

pub fn pad_to_c(pad: Pad) -> PfPad {
    PfPad {
        pleasure: pad.pleasure,
        arousal: pad.arousal,
        dominance: pad.dominance,
    }
}

pub fn lookup_to_c(lookup: RelationshipLookup) -> PfRelationshipLookup {
    match lookup {
        RelationshipLookup::Missing => PfRelationshipLookup {
            struct_size: expected_size::<PfRelationshipLookup>(),
            api_version: PF_ABI_VERSION,
            ..PfRelationshipLookup::default()
        },
        RelationshipLookup::Explicit { affinity, source } => PfRelationshipLookup {
            struct_size: expected_size::<PfRelationshipLookup>(),
            api_version: PF_ABI_VERSION,
            present: 1,
            source: relationship_layer_to_c(source),
            affinity: affinity.value(),
            ..PfRelationshipLookup::default()
        },
    }
}

pub fn evaluation_to_c(evaluation: EvaluationResult) -> PfEvaluationResult {
    PfEvaluationResult {
        concern: evaluation.concern(),
        effective_confidence: evaluation.effective_confidence().value(),
        raw_affinity_delta: evaluation.raw_affinity_delta(),
        raw_pad_delta: pad_to_c(evaluation.raw_pad_delta()),
        event_intensity: evaluation.event_intensity(),
        memory_salience: evaluation.memory_salience(),
        memory_kind: match evaluation.memory_decision() {
            personaflux_core::MemoryDecision::None => PF_MEMORY_NONE,
            personaflux_core::MemoryDecision::ShortTerm => PF_MEMORY_DECISION_SHORT_TERM,
            personaflux_core::MemoryDecision::LongTerm => PF_MEMORY_DECISION_LONG_TERM,
        },
    }
}

pub fn memory_to_c(record: MemoryRecord) -> PfMemoryRecord {
    PfMemoryRecord {
        struct_size: expected_size::<PfMemoryRecord>(),
        api_version: PF_ABI_VERSION,
        observer: record.observer().into_raw(),
        deed_id: record.deed_id(),
        actor: record.actor().into_raw(),
        target: record.target().map_or(0, MemberId::into_raw),
        has_target: u8::from(record.target().is_some()),
        impact: record.impact().value(),
        aggression: record.aggression().value(),
        salience: record.salience(),
        kind: memory_kind_to_c(record.kind()),
        created_tick: record.created_tick(),
        expires_at: record.expires_at(),
        ..PfMemoryRecord::default()
    }
}

pub fn outcome_to_c(outcome: DirectWitnessOutcome) -> PfDirectWitnessOutcome {
    let memory = outcome.memory().map(memory_to_c).unwrap_or_default();
    PfDirectWitnessOutcome {
        struct_size: expected_size::<PfDirectWitnessOutcome>(),
        api_version: PF_ABI_VERSION,
        deed_id: outcome.deed_id(),
        observer: outcome.observer().into_raw(),
        actor: outcome.actor().into_raw(),
        target: outcome.target().map_or(0, MemberId::into_raw),
        has_target: u8::from(outcome.target().is_some()),
        relationship: lookup_to_c(outcome.relationship()),
        evaluation: evaluation_to_c(outcome.evaluation()),
        previous_affinity: outcome.previous_affinity().value(),
        current_affinity: outcome.current_affinity().value(),
        previous_pad: pad_to_c(outcome.previous_pad()),
        current_pad: pad_to_c(outcome.current_pad()),
        has_memory: u8::from(outcome.memory().is_some()),
        memory,
        ..PfDirectWitnessOutcome::default()
    }
}

pub fn submission_to_c(submission: DirectWitnessSubmission) -> PfSubmissionResult {
    match submission {
        DirectWitnessSubmission::Applied(outcome) => PfSubmissionResult {
            struct_size: expected_size::<PfSubmissionResult>(),
            api_version: PF_ABI_VERSION,
            kind: PF_SUBMISSION_APPLIED,
            observer: outcome.observer().into_raw(),
            deed_id: outcome.deed_id(),
            outcome: outcome_to_c(outcome),
            ..PfSubmissionResult::default()
        },
        DirectWitnessSubmission::Duplicate { observer, deed_id } => PfSubmissionResult {
            struct_size: expected_size::<PfSubmissionResult>(),
            api_version: PF_ABI_VERSION,
            kind: PF_SUBMISSION_DUPLICATE,
            observer: observer.into_raw(),
            deed_id,
            ..PfSubmissionResult::default()
        },
    }
}

pub fn relationship_layer_to_c(layer: RelationshipLayer) -> u32 {
    match layer {
        RelationshipLayer::MemberToMember => PF_RELATIONSHIP_MEMBER_TO_MEMBER,
        RelationshipLayer::FactionToMember => PF_RELATIONSHIP_FACTION_TO_MEMBER,
        RelationshipLayer::FactionToFaction => PF_RELATIONSHIP_FACTION_TO_FACTION,
    }
}

pub fn memory_kind_to_c(kind: MemoryKind) -> u32 {
    match kind {
        MemoryKind::ShortTerm => PF_MEMORY_SHORT_TERM,
        MemoryKind::LongTerm => PF_MEMORY_LONG_TERM,
    }
}

pub fn event_to_c(event: SimulationEvent) -> PfEvent {
    let mut output = PfEvent {
        struct_size: expected_size::<PfEvent>(),
        api_version: PF_ABI_VERSION,
        ..PfEvent::default()
    };
    match event {
        SimulationEvent::FactionAdded { faction_id } => {
            output.kind = PF_EVENT_FACTION_ADDED;
            output.faction_id = faction_id.into_raw();
        }
        SimulationEvent::MemberAdded { member_id } => {
            output.kind = PF_EVENT_MEMBER_ADDED;
            output.member_id = member_id.into_raw();
        }
        SimulationEvent::RelationshipChanged {
            layer,
            subject,
            previous,
            current,
        } => {
            output.kind = PF_EVENT_RELATIONSHIP_CHANGED;
            output.relationship_layer = relationship_layer_to_c(layer);
            output.previous_present = u8::from(previous.is_some());
            output.current_present = u8::from(current.is_some());
            output.previous_affinity = previous.map_or(0.0, Affinity::value);
            output.current_affinity = current.map_or(0.0, Affinity::value);
            match subject {
                personaflux_core::RelationshipSubject::MemberToMember { observer, target } => {
                    output.relationship_source_id = observer.into_raw();
                    output.relationship_target_id = target.into_raw();
                }
                personaflux_core::RelationshipSubject::FactionToMember { faction, member } => {
                    output.relationship_source_id = faction.into_raw();
                    output.relationship_target_id = member.into_raw();
                }
                personaflux_core::RelationshipSubject::FactionToFaction { source, target } => {
                    output.relationship_source_id = source.into_raw();
                    output.relationship_target_id = target.into_raw();
                }
            }
        }
        SimulationEvent::DeedEvaluated {
            deed_id,
            observer,
            actor,
            target,
            evaluation,
        } => {
            output.kind = PF_EVENT_DEED_EVALUATED;
            output.deed_id = deed_id;
            output.observer = observer.into_raw();
            output.actor = actor.into_raw();
            output.target = target.map_or(0, MemberId::into_raw);
            output.has_target = u8::from(target.is_some());
            output.evaluation = evaluation_to_c(evaluation);
        }
        SimulationEvent::AffinityChanged {
            observer,
            actor,
            previous,
            current,
        } => {
            output.kind = PF_EVENT_AFFINITY_CHANGED;
            output.observer = observer.into_raw();
            output.actor = actor.into_raw();
            output.previous_affinity = previous.value();
            output.current_affinity = current.value();
        }
        SimulationEvent::PadChanged {
            member_id,
            previous,
            current,
        } => {
            output.kind = PF_EVENT_PAD_CHANGED;
            output.member_id = member_id.into_raw();
            output.previous_pad = pad_to_c(previous);
            output.current_pad = pad_to_c(current);
        }
        SimulationEvent::MemoryRemembered {
            observer,
            deed_id,
            kind,
            salience,
        } => {
            output.kind = PF_EVENT_MEMORY_REMEMBERED;
            output.observer = observer.into_raw();
            output.deed_id = deed_id;
            output.memory_kind = memory_kind_to_c(kind);
            output.salience = salience;
        }
        SimulationEvent::MemoryUpgraded {
            observer,
            deed_id,
            previous,
            current,
            salience,
        } => {
            output.kind = PF_EVENT_MEMORY_UPGRADED;
            output.observer = observer.into_raw();
            output.deed_id = deed_id;
            output.previous_memory_kind = memory_kind_to_c(previous);
            output.current_memory_kind = memory_kind_to_c(current);
            output.salience = salience;
        }
        SimulationEvent::MemoryExpired {
            observer,
            deed_id,
            kind,
        } => {
            output.kind = PF_EVENT_MEMORY_EXPIRED;
            output.observer = observer.into_raw();
            output.deed_id = deed_id;
            output.memory_kind = memory_kind_to_c(kind);
        }
        SimulationEvent::TimeAdvanced {
            previous_tick,
            current_tick,
        } => {
            output.kind = PF_EVENT_TIME_ADVANCED;
            output.previous_tick = previous_tick;
            output.current_tick = current_tick;
        }
    }
    output
}
