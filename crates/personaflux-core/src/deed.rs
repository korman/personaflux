use crate::evaluation::EvaluationResult;
use crate::pad::Pad;
use crate::relationship::RelationshipLookup;
use crate::simulation::MemberId;
use crate::values::{Affinity, Aggression, Impact};

/// A single direct-witness deed submitted to a simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectWitnessDeed {
    pub deed_id: u64,
    pub observer: MemberId,
    pub actor: MemberId,
    pub target: Option<MemberId>,
    pub impact: Impact,
    pub aggression: Aggression,
    pub threatens_observer: bool,
}

impl DirectWitnessDeed {
    pub const fn new(
        deed_id: u64,
        observer: MemberId,
        actor: MemberId,
        target: Option<MemberId>,
        impact: Impact,
        aggression: Aggression,
        threatens_observer: bool,
    ) -> Self {
        Self {
            deed_id,
            observer,
            actor,
            target,
            impact,
            aggression,
            threatens_observer,
        }
    }
}

/// Read-only result of applying one direct-witness deed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectWitnessOutcome {
    deed_id: u64,
    observer: MemberId,
    actor: MemberId,
    target: Option<MemberId>,
    relationship: RelationshipLookup,
    evaluation: EvaluationResult,
    previous_affinity: Affinity,
    current_affinity: Affinity,
    previous_pad: Pad,
    current_pad: Pad,
}

impl DirectWitnessOutcome {
    pub(crate) const fn new(
        deed: DirectWitnessDeed,
        relationship: RelationshipLookup,
        evaluation: EvaluationResult,
        previous_affinity: Affinity,
        current_affinity: Affinity,
        previous_pad: Pad,
        current_pad: Pad,
    ) -> Self {
        Self {
            deed_id: deed.deed_id,
            observer: deed.observer,
            actor: deed.actor,
            target: deed.target,
            relationship,
            evaluation,
            previous_affinity,
            current_affinity,
            previous_pad,
            current_pad,
        }
    }

    pub const fn deed_id(self) -> u64 {
        self.deed_id
    }

    pub const fn observer(self) -> MemberId {
        self.observer
    }

    pub const fn actor(self) -> MemberId {
        self.actor
    }

    pub const fn target(self) -> Option<MemberId> {
        self.target
    }

    pub const fn relationship(self) -> RelationshipLookup {
        self.relationship
    }

    pub const fn relationship_lookup(self) -> RelationshipLookup {
        self.relationship()
    }

    pub const fn evaluation(self) -> EvaluationResult {
        self.evaluation
    }

    pub const fn previous_affinity(self) -> Affinity {
        self.previous_affinity
    }

    pub const fn current_affinity(self) -> Affinity {
        self.current_affinity
    }

    pub const fn previous_pad(self) -> Pad {
        self.previous_pad
    }

    pub const fn current_pad(self) -> Pad {
        self.current_pad
    }
}
