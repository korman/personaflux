use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::bounds::{apply_affinity_delta, apply_bounded_delta};
use crate::deed::{BatchError, DirectWitnessDeed, DirectWitnessOutcome, DirectWitnessSubmission};
use crate::evaluation::{DirectWitnessInput, EvaluationResult, evaluate_direct_witness};
use crate::memory::{MemoryChange, MemoryKind, MemoryRecord, MemoryStore};
use crate::pad::Pad;
use crate::relationship::{
    RelationshipLayer, RelationshipLookup, RelationshipStore, RelationshipSubject,
};
use crate::values::{Affinity, ValueError};

/// Stable identifier for a faction inside one simulation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactionId(u64);

/// Stable identifier for a member inside one simulation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MemberId(u64);

impl MemberId {
    #[cfg(test)]
    pub(crate) const fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

/// Configuration shared by one simulation instance.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimulationConfig {
    pub random_seed: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    EmptyName,
    FactionNotFound(FactionId),
    MemberNotFound(MemberId),
    ActorTargetSame(MemberId),
    Value(ValueError),
}

impl From<ValueError> for Error {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SimulationEvent {
    FactionAdded {
        faction_id: FactionId,
    },
    MemberAdded {
        member_id: MemberId,
    },
    RelationshipChanged {
        layer: RelationshipLayer,
        subject: RelationshipSubject,
        previous: Option<crate::Affinity>,
        current: Option<crate::Affinity>,
    },
    DeedEvaluated {
        deed_id: u64,
        observer: MemberId,
        actor: MemberId,
        target: Option<MemberId>,
        evaluation: EvaluationResult,
    },
    AffinityChanged {
        observer: MemberId,
        actor: MemberId,
        previous: Affinity,
        current: Affinity,
    },
    PadChanged {
        member_id: MemberId,
        previous: Pad,
        current: Pad,
    },
    MemoryRemembered {
        observer: MemberId,
        deed_id: u64,
        kind: MemoryKind,
        salience: f32,
    },
    MemoryUpgraded {
        observer: MemberId,
        deed_id: u64,
        previous: MemoryKind,
        current: MemoryKind,
        salience: f32,
    },
}

#[derive(Clone, Debug)]
struct Faction {
    name: String,
}

#[derive(Clone, Debug)]
struct Member {
    faction_id: FactionId,
    pad: Pad,
}

/// Owns all state for an isolated social-affect simulation.
pub struct Simulation {
    config: SimulationConfig,
    next_faction_id: u64,
    next_member_id: u64,
    factions: BTreeMap<FactionId, Faction>,
    members: BTreeMap<MemberId, Member>,
    relationships: RelationshipStore,
    dynamic_affinities: BTreeMap<(MemberId, MemberId), Affinity>,
    processed_deeds: BTreeSet<(MemberId, u64)>,
    memories: MemoryStore,
    events: VecDeque<SimulationEvent>,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            config,
            next_faction_id: 1,
            next_member_id: 1,
            factions: BTreeMap::new(),
            members: BTreeMap::new(),
            relationships: RelationshipStore::default(),
            dynamic_affinities: BTreeMap::new(),
            processed_deeds: BTreeSet::new(),
            memories: MemoryStore::default(),
            events: VecDeque::new(),
        }
    }

    pub fn config(&self) -> SimulationConfig {
        self.config
    }

    pub fn add_faction(&mut self, name: impl Into<String>) -> Result<FactionId, Error> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(Error::EmptyName);
        }

        let faction_id = FactionId(self.next_faction_id);
        self.next_faction_id += 1;
        self.factions.insert(faction_id, Faction { name });
        self.events
            .push_back(SimulationEvent::FactionAdded { faction_id });
        Ok(faction_id)
    }

    pub fn faction_name(&self, faction_id: FactionId) -> Option<&str> {
        self.factions
            .get(&faction_id)
            .map(|faction| faction.name.as_str())
    }

    pub fn add_member(&mut self, faction_id: FactionId) -> Result<MemberId, Error> {
        if !self.factions.contains_key(&faction_id) {
            return Err(Error::FactionNotFound(faction_id));
        }

        let member_id = MemberId(self.next_member_id);
        self.next_member_id += 1;
        self.members.insert(
            member_id,
            Member {
                faction_id,
                pad: Pad::default(),
            },
        );
        self.events
            .push_back(SimulationEvent::MemberAdded { member_id });
        Ok(member_id)
    }

    pub fn member_faction(&self, member_id: MemberId) -> Option<FactionId> {
        self.members.get(&member_id).map(|member| member.faction_id)
    }

    pub fn member_pad(&self, member_id: MemberId) -> Option<Pad> {
        self.members.get(&member_id).map(|member| member.pad)
    }

    /// Returns the accumulated affinity from one observer toward one actor.
    ///
    /// Dynamic affinity is separate from configured relationship layers and
    /// starts at neutral when the pair has not been updated yet.
    pub fn member_affinity(&self, observer: MemberId, actor: MemberId) -> Result<Affinity, Error> {
        self.require_member(observer)?;
        self.require_member(actor)?;
        Ok(self
            .dynamic_affinities
            .get(&(observer, actor))
            .copied()
            .unwrap_or_else(|| Affinity::new(0.0).expect("neutral affinity is valid")))
    }

    /// Returns one remembered deed for an observer, if it exists.
    pub fn memory(&self, observer: MemberId, deed_id: u64) -> Result<Option<MemoryRecord>, Error> {
        self.require_member(observer)?;
        Ok(self.memories.get(observer, deed_id))
    }

    /// Returns all remembered deeds for an observer in deed-id order.
    pub fn memories_for(&self, observer: MemberId) -> Result<Vec<MemoryRecord>, Error> {
        self.require_member(observer)?;
        Ok(self.memories.for_observer(observer))
    }

    /// Applies one direct-witness deed atomically to the observing member.
    pub fn submit_direct_witness(
        &mut self,
        deed: DirectWitnessDeed,
    ) -> Result<DirectWitnessSubmission, Error> {
        match self.submit_direct_witness_batch(std::slice::from_ref(&deed)) {
            Ok(mut results) => Ok(results.remove(0)),
            Err(error) => Err(error.into_error()),
        }
    }

    /// Applies direct-witness deeds in input order as one atomic transaction.
    pub fn submit_direct_witness_batch(
        &mut self,
        deeds: &[DirectWitnessDeed],
    ) -> Result<Vec<DirectWitnessSubmission>, BatchError> {
        for (index, deed) in deeds.iter().enumerate() {
            self.validate_deed(*deed)
                .map_err(|error| BatchError::new(index, error))?;
        }

        let mut members = self.members.clone();
        let mut dynamic_affinities = self.dynamic_affinities.clone();
        let mut processed_deeds = self.processed_deeds.clone();
        let mut memories = self.memories.clone();
        let mut pending_events = Vec::new();
        let mut results = Vec::with_capacity(deeds.len());

        for (index, deed) in deeds.iter().copied().enumerate() {
            let result = self
                .apply_deed_to_state(
                    deed,
                    &mut members,
                    &mut dynamic_affinities,
                    &mut processed_deeds,
                    &mut memories,
                    &mut pending_events,
                )
                .map_err(|error| BatchError::new(index, error))?;
            results.push(result);
        }

        self.members = members;
        self.dynamic_affinities = dynamic_affinities;
        self.processed_deeds = processed_deeds;
        self.memories = memories;
        self.events.extend(pending_events);
        Ok(results)
    }

    fn validate_deed(&self, deed: DirectWitnessDeed) -> Result<(), Error> {
        self.require_member(deed.observer)?;
        self.require_member(deed.actor)?;
        if let Some(target) = deed.target {
            self.require_member(target)?;
            if deed.actor == target {
                return Err(Error::ActorTargetSame(target));
            }
        }
        Ok(())
    }

    fn apply_deed_to_state(
        &self,
        deed: DirectWitnessDeed,
        members: &mut BTreeMap<MemberId, Member>,
        dynamic_affinities: &mut BTreeMap<(MemberId, MemberId), Affinity>,
        processed_deeds: &mut BTreeSet<(MemberId, u64)>,
        memories: &mut MemoryStore,
        pending_events: &mut Vec<SimulationEvent>,
    ) -> Result<DirectWitnessSubmission, Error> {
        let key = (deed.observer, deed.deed_id);
        if processed_deeds.contains(&key) {
            return Ok(DirectWitnessSubmission::Duplicate {
                observer: deed.observer,
                deed_id: deed.deed_id,
            });
        }

        let relationship = match deed.target {
            Some(target) => self.effective_member_relationship(deed.observer, target)?,
            None => RelationshipLookup::Missing,
        };
        let target_affinity = relationship
            .affinity()
            .unwrap_or_else(|| Affinity::new(0.0).expect("neutral affinity is valid"));
        let evaluation = evaluate_direct_witness(DirectWitnessInput::new(
            deed.impact,
            deed.aggression,
            target_affinity,
            deed.target.is_some(),
            deed.threatens_observer,
        ));

        let previous_affinity = dynamic_affinities
            .get(&(deed.observer, deed.actor))
            .copied()
            .unwrap_or_else(|| Affinity::new(0.0).expect("neutral affinity is valid"));
        let current_affinity =
            apply_affinity_delta(previous_affinity, evaluation.raw_affinity_delta())?;
        let previous_pad = members
            .get(&deed.observer)
            .expect("observer was validated")
            .pad;
        let raw_pad = evaluation.raw_pad_delta();
        let current_pad = Pad::new(
            apply_bounded_delta(previous_pad.pleasure, raw_pad.pleasure)?,
            apply_bounded_delta(previous_pad.arousal, raw_pad.arousal)?,
            apply_bounded_delta(previous_pad.dominance, raw_pad.dominance)?,
        )?;

        let memory_change = MemoryRecord::from_evaluation(deed, evaluation)
            .map(|record| memories.insert_or_upgrade(record))
            .unwrap_or(MemoryChange::None);
        let (memory, memory_event) = match memory_change {
            MemoryChange::None => (None, None),
            MemoryChange::Remembered(record) => (
                Some(record),
                Some(SimulationEvent::MemoryRemembered {
                    observer: record.observer(),
                    deed_id: record.deed_id(),
                    kind: record.kind(),
                    salience: record.salience(),
                }),
            ),
            MemoryChange::Upgraded { previous, current } => (
                Some(current),
                Some(SimulationEvent::MemoryUpgraded {
                    observer: current.observer(),
                    deed_id: current.deed_id(),
                    previous,
                    current: current.kind(),
                    salience: current.salience(),
                }),
            ),
        };
        let outcome = DirectWitnessOutcome::new(
            deed,
            relationship,
            evaluation,
            previous_affinity,
            current_affinity,
            previous_pad,
            current_pad,
        )
        .with_memory(memory);
        processed_deeds.insert(key);
        if current_affinity != previous_affinity {
            dynamic_affinities.insert((deed.observer, deed.actor), current_affinity);
        }
        if current_pad != previous_pad {
            members
                .get_mut(&deed.observer)
                .expect("observer was validated")
                .pad = current_pad;
        }

        pending_events.push(SimulationEvent::DeedEvaluated {
            deed_id: deed.deed_id,
            observer: deed.observer,
            actor: deed.actor,
            target: deed.target,
            evaluation,
        });
        if current_affinity != previous_affinity {
            pending_events.push(SimulationEvent::AffinityChanged {
                observer: deed.observer,
                actor: deed.actor,
                previous: previous_affinity,
                current: current_affinity,
            });
        }
        if current_pad != previous_pad {
            pending_events.push(SimulationEvent::PadChanged {
                member_id: deed.observer,
                previous: previous_pad,
                current: current_pad,
            });
        }
        if let Some(event) = memory_event {
            pending_events.push(event);
        }
        Ok(DirectWitnessSubmission::Applied(outcome))
    }

    pub fn set_member_relationship(
        &mut self,
        observer: MemberId,
        target: MemberId,
        affinity: crate::Affinity,
    ) -> Result<(), Error> {
        self.require_member(observer)?;
        self.require_member(target)?;

        let previous = self
            .relationships
            .set_member_to_member(observer, target, affinity);
        if previous != Some(affinity) {
            self.events.push_back(SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::MemberToMember,
                subject: RelationshipSubject::MemberToMember { observer, target },
                previous,
                current: Some(affinity),
            });
        }
        Ok(())
    }

    pub fn clear_member_relationship(
        &mut self,
        observer: MemberId,
        target: MemberId,
    ) -> Result<(), Error> {
        self.require_member(observer)?;
        self.require_member(target)?;

        let previous = self.relationships.clear_member_to_member(observer, target);
        if previous.is_some() {
            self.events.push_back(SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::MemberToMember,
                subject: RelationshipSubject::MemberToMember { observer, target },
                previous,
                current: None,
            });
        }
        Ok(())
    }

    pub fn set_faction_member_relationship(
        &mut self,
        faction: FactionId,
        member: MemberId,
        affinity: crate::Affinity,
    ) -> Result<(), Error> {
        self.require_faction(faction)?;
        self.require_member(member)?;

        let previous = self
            .relationships
            .set_faction_to_member(faction, member, affinity);
        if previous != Some(affinity) {
            self.events.push_back(SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::FactionToMember,
                subject: RelationshipSubject::FactionToMember { faction, member },
                previous,
                current: Some(affinity),
            });
        }
        Ok(())
    }

    pub fn clear_faction_member_relationship(
        &mut self,
        faction: FactionId,
        member: MemberId,
    ) -> Result<(), Error> {
        self.require_faction(faction)?;
        self.require_member(member)?;

        let previous = self.relationships.clear_faction_to_member(faction, member);
        if previous.is_some() {
            self.events.push_back(SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::FactionToMember,
                subject: RelationshipSubject::FactionToMember { faction, member },
                previous,
                current: None,
            });
        }
        Ok(())
    }

    pub fn set_faction_relationship(
        &mut self,
        source: FactionId,
        target: FactionId,
        affinity: crate::Affinity,
    ) -> Result<(), Error> {
        self.require_faction(source)?;
        self.require_faction(target)?;

        let previous = self
            .relationships
            .set_faction_to_faction(source, target, affinity);
        if previous != Some(affinity) {
            self.events.push_back(SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::FactionToFaction,
                subject: RelationshipSubject::FactionToFaction { source, target },
                previous,
                current: Some(affinity),
            });
        }
        Ok(())
    }

    pub fn clear_faction_relationship(
        &mut self,
        source: FactionId,
        target: FactionId,
    ) -> Result<(), Error> {
        self.require_faction(source)?;
        self.require_faction(target)?;

        let previous = self.relationships.clear_faction_to_faction(source, target);
        if previous.is_some() {
            self.events.push_back(SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::FactionToFaction,
                subject: RelationshipSubject::FactionToFaction { source, target },
                previous,
                current: None,
            });
        }
        Ok(())
    }

    pub fn member_relationship(
        &self,
        observer: MemberId,
        target: MemberId,
    ) -> Result<RelationshipLookup, Error> {
        self.require_member(observer)?;
        self.require_member(target)?;
        Ok(self.relationships.member_to_member(observer, target))
    }

    pub fn faction_member_relationship(
        &self,
        faction: FactionId,
        member: MemberId,
    ) -> Result<RelationshipLookup, Error> {
        self.require_faction(faction)?;
        self.require_member(member)?;
        Ok(self.relationships.faction_to_member(faction, member))
    }

    pub fn faction_relationship(
        &self,
        source: FactionId,
        target: FactionId,
    ) -> Result<RelationshipLookup, Error> {
        self.require_faction(source)?;
        self.require_faction(target)?;
        Ok(self.relationships.faction_to_faction(source, target))
    }

    pub fn effective_member_relationship(
        &self,
        observer: MemberId,
        target: MemberId,
    ) -> Result<RelationshipLookup, Error> {
        let observer_faction = self.member_faction_or_error(observer)?;
        let target_faction = self.member_faction_or_error(target)?;

        let member_relationship = self.relationships.member_to_member(observer, target);
        if !matches!(member_relationship, RelationshipLookup::Missing) {
            return Ok(member_relationship);
        }

        let faction_member_relationship = self
            .relationships
            .faction_to_member(observer_faction, target);
        if !matches!(faction_member_relationship, RelationshipLookup::Missing) {
            return Ok(faction_member_relationship);
        }

        Ok(self
            .relationships
            .faction_to_faction(observer_faction, target_faction))
    }

    fn require_faction(&self, faction_id: FactionId) -> Result<(), Error> {
        if self.factions.contains_key(&faction_id) {
            Ok(())
        } else {
            Err(Error::FactionNotFound(faction_id))
        }
    }

    fn require_member(&self, member_id: MemberId) -> Result<(), Error> {
        if self.members.contains_key(&member_id) {
            Ok(())
        } else {
            Err(Error::MemberNotFound(member_id))
        }
    }

    fn member_faction_or_error(&self, member_id: MemberId) -> Result<FactionId, Error> {
        self.members
            .get(&member_id)
            .map(|member| member.faction_id)
            .ok_or(Error::MemberNotFound(member_id))
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = SimulationEvent> + '_ {
        self.events.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Affinity, Aggression, DirectWitnessDeed, DirectWitnessSubmission, Impact, MemoryKind,
        RelationshipLookup,
    };

    fn affinity(value: f32) -> Affinity {
        Affinity::new(value).unwrap()
    }

    fn impact(value: f32) -> Impact {
        Impact::new(value).unwrap()
    }

    fn aggression(value: f32) -> Aggression {
        Aggression::new(value).unwrap()
    }

    fn relationship_simulation() -> (Simulation, FactionId, FactionId, MemberId, MemberId) {
        let mut simulation = Simulation::new(SimulationConfig::default());
        let faction_a = simulation.add_faction("A").unwrap();
        let faction_b = simulation.add_faction("B").unwrap();
        let member_a = simulation.add_member(faction_a).unwrap();
        let member_b = simulation.add_member(faction_b).unwrap();
        simulation.drain_events().for_each(drop);
        (simulation, faction_a, faction_b, member_a, member_b)
    }

    fn deed(
        deed_id: u64,
        observer: MemberId,
        actor: MemberId,
        target: Option<MemberId>,
        impact_value: f32,
        aggression_value: f32,
        threatens_observer: bool,
    ) -> DirectWitnessDeed {
        DirectWitnessDeed::new(
            deed_id,
            observer,
            actor,
            target,
            impact(impact_value),
            aggression(aggression_value),
            threatens_observer,
        )
    }

    fn applied(submission: DirectWitnessSubmission) -> DirectWitnessOutcome {
        match submission {
            DirectWitnessSubmission::Applied(outcome) => outcome,
            DirectWitnessSubmission::Duplicate { .. } => panic!("expected applied submission"),
        }
    }

    #[test]
    fn adds_factions_and_members_with_stable_events() {
        let mut simulation = Simulation::new(SimulationConfig::default());
        let faction_id = simulation.add_faction("Settlers").unwrap();
        let member_id = simulation.add_member(faction_id).unwrap();

        assert_eq!(simulation.faction_name(faction_id), Some("Settlers"));
        assert_eq!(simulation.member_faction(member_id), Some(faction_id));
        assert_eq!(simulation.member_pad(member_id), Some(Pad::default()));
        assert_eq!(
            simulation.drain_events().collect::<Vec<_>>(),
            vec![
                SimulationEvent::FactionAdded { faction_id },
                SimulationEvent::MemberAdded { member_id },
            ]
        );
    }

    #[test]
    fn rejects_members_for_unknown_factions() {
        let mut simulation = Simulation::new(SimulationConfig::default());
        let unknown_faction = FactionId(42);

        assert_eq!(
            simulation.add_member(unknown_faction),
            Err(Error::FactionNotFound(unknown_faction))
        );
    }

    #[test]
    fn member_relationships_are_directional_and_explicit_zero_is_not_missing() {
        let (mut simulation, _, _, member_a, member_b) = relationship_simulation();

        simulation
            .set_member_relationship(member_a, member_b, affinity(0.7))
            .unwrap();
        simulation
            .set_member_relationship(member_b, member_a, affinity(-0.2))
            .unwrap();

        assert_eq!(
            simulation.member_relationship(member_a, member_b).unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.7),
                source: RelationshipLayer::MemberToMember,
            }
        );
        assert_eq!(
            simulation.member_relationship(member_b, member_a).unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(-0.2),
                source: RelationshipLayer::MemberToMember,
            }
        );

        simulation
            .set_member_relationship(member_a, member_b, affinity(0.0))
            .unwrap();
        assert_eq!(
            simulation.member_relationship(member_a, member_b).unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.0),
                source: RelationshipLayer::MemberToMember,
            }
        );
    }

    #[test]
    fn effective_relationship_uses_member_then_faction_member_then_faction_fallbacks() {
        let (mut simulation, faction_a, faction_b, member_a, member_b) = relationship_simulation();

        assert_eq!(
            simulation
                .effective_member_relationship(member_a, member_b)
                .unwrap(),
            RelationshipLookup::Missing
        );

        simulation
            .set_faction_relationship(faction_a, faction_b, affinity(0.2))
            .unwrap();
        assert_eq!(
            simulation
                .effective_member_relationship(member_a, member_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.2),
                source: RelationshipLayer::FactionToFaction,
            }
        );

        simulation
            .set_faction_member_relationship(faction_a, member_b, affinity(0.4))
            .unwrap();
        assert_eq!(
            simulation
                .effective_member_relationship(member_a, member_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.4),
                source: RelationshipLayer::FactionToMember,
            }
        );

        simulation
            .set_member_relationship(member_a, member_b, affinity(-0.6))
            .unwrap();
        assert_eq!(
            simulation
                .effective_member_relationship(member_a, member_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(-0.6),
                source: RelationshipLayer::MemberToMember,
            }
        );

        simulation
            .clear_member_relationship(member_a, member_b)
            .unwrap();
        simulation
            .clear_faction_member_relationship(faction_a, member_b)
            .unwrap();
        assert_eq!(
            simulation
                .effective_member_relationship(member_a, member_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.2),
                source: RelationshipLayer::FactionToFaction,
            }
        );
    }

    #[test]
    fn relationship_changes_emit_only_for_actual_changes() {
        let (mut simulation, faction_a, faction_b, member_a, member_b) = relationship_simulation();

        simulation
            .set_member_relationship(member_a, member_b, affinity(0.5))
            .unwrap();
        assert_eq!(
            simulation.drain_events().collect::<Vec<_>>(),
            vec![SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::MemberToMember,
                subject: RelationshipSubject::MemberToMember {
                    observer: member_a,
                    target: member_b,
                },
                previous: None,
                current: Some(affinity(0.5)),
            }]
        );

        simulation
            .set_member_relationship(member_a, member_b, affinity(0.5))
            .unwrap();
        assert_eq!(simulation.drain_events().next(), None);

        simulation
            .set_member_relationship(member_a, member_b, affinity(0.8))
            .unwrap();
        simulation
            .clear_member_relationship(member_a, member_b)
            .unwrap();
        assert_eq!(
            simulation.drain_events().collect::<Vec<_>>(),
            vec![
                SimulationEvent::RelationshipChanged {
                    layer: RelationshipLayer::MemberToMember,
                    subject: RelationshipSubject::MemberToMember {
                        observer: member_a,
                        target: member_b,
                    },
                    previous: Some(affinity(0.5)),
                    current: Some(affinity(0.8)),
                },
                SimulationEvent::RelationshipChanged {
                    layer: RelationshipLayer::MemberToMember,
                    subject: RelationshipSubject::MemberToMember {
                        observer: member_a,
                        target: member_b,
                    },
                    previous: Some(affinity(0.8)),
                    current: None,
                },
            ]
        );

        simulation
            .clear_member_relationship(member_a, member_b)
            .unwrap();
        assert_eq!(simulation.drain_events().next(), None);

        simulation
            .set_faction_relationship(faction_a, faction_b, affinity(0.1))
            .unwrap();
        assert_eq!(
            simulation.drain_events().collect::<Vec<_>>(),
            vec![SimulationEvent::RelationshipChanged {
                layer: RelationshipLayer::FactionToFaction,
                subject: RelationshipSubject::FactionToFaction {
                    source: faction_a,
                    target: faction_b,
                },
                previous: None,
                current: Some(affinity(0.1)),
            }]
        );
    }

    #[test]
    fn each_relationship_layer_supports_direct_lookup_override_and_clear() {
        let (mut simulation, faction_a, faction_b, member_a, member_b) = relationship_simulation();

        simulation
            .set_member_relationship(member_a, member_b, affinity(0.1))
            .unwrap();
        simulation
            .set_faction_member_relationship(faction_a, member_b, affinity(0.2))
            .unwrap();
        simulation
            .set_faction_relationship(faction_a, faction_b, affinity(0.3))
            .unwrap();
        simulation.drain_events().for_each(drop);

        assert_eq!(
            simulation.member_relationship(member_a, member_b).unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.1),
                source: RelationshipLayer::MemberToMember,
            }
        );
        assert_eq!(
            simulation
                .faction_member_relationship(faction_a, member_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.2),
                source: RelationshipLayer::FactionToMember,
            }
        );
        assert_eq!(
            simulation
                .faction_relationship(faction_a, faction_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.3),
                source: RelationshipLayer::FactionToFaction,
            }
        );

        simulation
            .set_member_relationship(member_a, member_b, affinity(-0.1))
            .unwrap();
        simulation
            .set_faction_member_relationship(faction_a, member_b, affinity(-0.2))
            .unwrap();
        simulation
            .set_faction_relationship(faction_a, faction_b, affinity(-0.3))
            .unwrap();
        assert_eq!(
            simulation.member_relationship(member_a, member_b).unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(-0.1),
                source: RelationshipLayer::MemberToMember,
            }
        );
        assert_eq!(
            simulation
                .faction_member_relationship(faction_a, member_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(-0.2),
                source: RelationshipLayer::FactionToMember,
            }
        );
        assert_eq!(
            simulation
                .faction_relationship(faction_a, faction_b)
                .unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(-0.3),
                source: RelationshipLayer::FactionToFaction,
            }
        );

        simulation
            .clear_member_relationship(member_a, member_b)
            .unwrap();
        simulation
            .clear_faction_member_relationship(faction_a, member_b)
            .unwrap();
        simulation
            .clear_faction_relationship(faction_a, faction_b)
            .unwrap();
        assert_eq!(
            simulation.member_relationship(member_a, member_b).unwrap(),
            RelationshipLookup::Missing
        );
        assert_eq!(
            simulation
                .faction_member_relationship(faction_a, member_b)
                .unwrap(),
            RelationshipLookup::Missing
        );
        assert_eq!(
            simulation
                .faction_relationship(faction_a, faction_b)
                .unwrap(),
            RelationshipLookup::Missing
        );
    }

    #[test]
    fn relationship_operations_reject_unknown_entities_but_allow_self_relationships() {
        let (mut simulation, faction_a, _, member_a, _) = relationship_simulation();
        let unknown_member = MemberId(99);
        let unknown_faction = FactionId(99);

        assert_eq!(
            simulation.set_member_relationship(member_a, unknown_member, affinity(0.1)),
            Err(Error::MemberNotFound(unknown_member))
        );
        assert_eq!(
            simulation.set_faction_member_relationship(unknown_faction, member_a, affinity(0.1)),
            Err(Error::FactionNotFound(unknown_faction))
        );
        assert_eq!(
            simulation.set_faction_relationship(faction_a, unknown_faction, affinity(0.1)),
            Err(Error::FactionNotFound(unknown_faction))
        );
        assert_eq!(
            simulation.member_relationship(member_a, unknown_member),
            Err(Error::MemberNotFound(unknown_member))
        );

        simulation
            .set_member_relationship(member_a, member_a, affinity(0.3))
            .unwrap();
        assert_eq!(
            simulation.member_relationship(member_a, member_a).unwrap(),
            RelationshipLookup::Explicit {
                affinity: affinity(0.3),
                source: RelationshipLayer::MemberToMember,
            }
        );
    }

    #[test]
    fn direct_witness_updates_dynamic_affinity_and_observer_pad_atomically() {
        let (mut simulation, faction_a, _, observer, actor) = relationship_simulation();
        let target = simulation.add_member(faction_a).unwrap();
        simulation
            .set_member_relationship(observer, target, affinity(1.0))
            .unwrap();
        simulation.drain_events().for_each(drop);

        let outcome = applied(
            simulation
                .submit_direct_witness(deed(7, observer, actor, Some(target), 0.8, 0.2, false))
                .unwrap(),
        );

        assert_eq!(outcome.deed_id(), 7);
        assert_eq!(outcome.observer(), observer);
        assert_eq!(outcome.actor(), actor);
        assert_eq!(outcome.target(), Some(target));
        assert_eq!(
            outcome.relationship().source(),
            Some(RelationshipLayer::MemberToMember)
        );
        assert_eq!(outcome.previous_affinity(), affinity(0.0));
        assert!((outcome.current_affinity().value() - 0.8).abs() < 1e-6);
        assert_eq!(outcome.previous_pad(), Pad::default());
        assert!((outcome.current_pad().pleasure - 0.4).abs() < 1e-6);
        assert!((outcome.current_pad().arousal - 0.4).abs() < 1e-6);
        assert_eq!(
            simulation.member_affinity(observer, actor).unwrap(),
            outcome.current_affinity()
        );
        assert_eq!(simulation.member_pad(observer), Some(outcome.current_pad()));

        let events = simulation.drain_events().collect::<Vec<_>>();
        assert!(matches!(
            events[0],
            SimulationEvent::DeedEvaluated { deed_id: 7, .. }
        ));
        assert!(
            matches!(events[1], SimulationEvent::AffinityChanged { observer: id, actor: actor_id, .. } if id == observer && actor_id == actor)
        );
        assert!(
            matches!(events[2], SimulationEvent::PadChanged { member_id, .. } if member_id == observer)
        );
    }

    #[test]
    fn direct_witness_uses_relationship_fallback_but_keeps_dynamic_affinity_separate() {
        let (mut simulation, faction_a, faction_b, observer, actor) = relationship_simulation();
        let target = simulation.add_member(faction_b).unwrap();
        simulation
            .set_faction_relationship(faction_a, faction_b, affinity(-1.0))
            .unwrap();
        simulation.drain_events().for_each(drop);

        let outcome = applied(
            simulation
                .submit_direct_witness(deed(1, observer, actor, Some(target), 0.5, 0.0, false))
                .unwrap(),
        );

        assert_eq!(
            outcome.relationship().source(),
            Some(RelationshipLayer::FactionToFaction)
        );
        assert!(outcome.evaluation().raw_affinity_delta() < 0.0);
        assert_eq!(
            simulation.member_relationship(observer, actor).unwrap(),
            RelationshipLookup::Missing
        );
        assert!(simulation.member_affinity(observer, actor).unwrap().value() < 0.0);
    }

    #[test]
    fn direct_witness_zero_impact_changes_pad_but_not_affinity() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let outcome = applied(
            simulation
                .submit_direct_witness(deed(2, observer, actor, None, 0.0, 0.6, true))
                .unwrap(),
        );

        assert_eq!(outcome.current_affinity(), affinity(0.0));
        assert_eq!(outcome.current_pad().pleasure, 0.0);
        assert!((outcome.current_pad().arousal - 0.3).abs() < 1e-6);
        assert_eq!(outcome.current_pad().dominance, 0.0);
        let events = simulation.drain_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 3);
        assert!(matches!(
            events[0],
            SimulationEvent::DeedEvaluated { target: None, .. }
        ));
        assert!(matches!(events[1], SimulationEvent::PadChanged { .. }));
        assert!(matches!(
            events[2],
            SimulationEvent::MemoryRemembered {
                kind: MemoryKind::ShortTerm,
                ..
            }
        ));
    }

    #[test]
    fn direct_witness_with_no_effect_emits_only_evaluation_event() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let outcome = applied(
            simulation
                .submit_direct_witness(deed(22, observer, actor, None, 0.0, 0.0, true))
                .unwrap(),
        );

        assert_eq!(outcome.previous_affinity(), outcome.current_affinity());
        assert_eq!(outcome.previous_pad(), outcome.current_pad());
        assert_eq!(
            simulation.drain_events().collect::<Vec<_>>(),
            vec![SimulationEvent::DeedEvaluated {
                deed_id: 22,
                observer,
                actor,
                target: None,
                evaluation: outcome.evaluation(),
            }]
        );
    }

    #[test]
    fn direct_witness_rejects_actor_target_self_reference_without_changes() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let before_pad = simulation.member_pad(observer).unwrap();

        assert_eq!(
            simulation.submit_direct_witness(deed(
                3,
                observer,
                actor,
                Some(actor),
                0.2,
                0.1,
                false
            )),
            Err(Error::ActorTargetSame(actor))
        );
        assert_eq!(
            simulation.member_affinity(observer, actor).unwrap(),
            affinity(0.0)
        );
        assert_eq!(simulation.member_pad(observer), Some(before_pad));
        assert!(simulation.drain_events().next().is_none());
    }

    #[test]
    fn direct_witness_allows_observer_to_be_actor() {
        let (mut simulation, _, _, observer, _) = relationship_simulation();
        let outcome = applied(
            simulation
                .submit_direct_witness(deed(4, observer, observer, None, 0.2, 0.0, false))
                .unwrap(),
        );
        assert_eq!(outcome.observer(), observer);
        assert_eq!(outcome.actor(), observer);
        assert!(
            simulation
                .member_affinity(observer, observer)
                .unwrap()
                .value()
                > 0.0
        );
    }

    #[test]
    fn direct_witness_failure_preserves_existing_event_queue() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        simulation
            .set_member_relationship(observer, actor, affinity(0.5))
            .unwrap();
        assert_eq!(
            simulation.submit_direct_witness(deed(
                5,
                observer,
                actor,
                Some(MemberId(999)),
                0.2,
                0.0,
                false
            )),
            Err(Error::MemberNotFound(MemberId(999)))
        );
        let events = simulation.drain_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            SimulationEvent::RelationshipChanged { .. }
        ));
        assert_eq!(
            simulation.member_affinity(observer, actor).unwrap(),
            affinity(0.0)
        );
        assert_eq!(simulation.member_pad(observer), Some(Pad::default()));
    }

    #[test]
    fn direct_witness_resolves_every_relationship_source_and_missing() {
        let (mut simulation, faction_a, faction_b, observer, actor) = relationship_simulation();
        let target = simulation.add_member(faction_b).unwrap();
        simulation.drain_events().for_each(drop);

        let missing = applied(
            simulation
                .submit_direct_witness(deed(10, observer, actor, Some(target), 0.1, 0.0, false))
                .unwrap(),
        );
        assert_eq!(missing.relationship(), RelationshipLookup::Missing);

        simulation
            .set_faction_relationship(faction_a, faction_b, affinity(0.2))
            .unwrap();
        let faction = applied(
            simulation
                .submit_direct_witness(deed(11, observer, actor, Some(target), 0.1, 0.0, false))
                .unwrap(),
        );
        assert_eq!(
            faction.relationship().source(),
            Some(RelationshipLayer::FactionToFaction)
        );

        simulation
            .set_faction_member_relationship(faction_a, target, affinity(0.4))
            .unwrap();
        let faction_member = applied(
            simulation
                .submit_direct_witness(deed(12, observer, actor, Some(target), 0.1, 0.0, false))
                .unwrap(),
        );
        assert_eq!(
            faction_member.relationship().source(),
            Some(RelationshipLayer::FactionToMember)
        );

        simulation
            .set_member_relationship(observer, target, affinity(0.6))
            .unwrap();
        let member = applied(
            simulation
                .submit_direct_witness(deed(13, observer, actor, Some(target), 0.1, 0.0, false))
                .unwrap(),
        );
        assert_eq!(
            member.relationship().source(),
            Some(RelationshipLayer::MemberToMember)
        );
    }

    #[test]
    fn direct_threat_changes_dominance_but_third_party_attack_does_not() {
        let (mut simulation, faction_a, _, observer, actor) = relationship_simulation();
        let third_party = simulation.add_member(faction_a).unwrap();
        simulation.drain_events().for_each(drop);

        let threatened = applied(
            simulation
                .submit_direct_witness(deed(20, observer, actor, Some(observer), 0.0, 0.5, true))
                .unwrap(),
        );
        assert!((threatened.current_pad().dominance + 0.2).abs() < 1e-6);

        let third_party_attack = applied(
            simulation
                .submit_direct_witness(deed(
                    21,
                    observer,
                    actor,
                    Some(third_party),
                    0.0,
                    0.5,
                    false,
                ))
                .unwrap(),
        );
        assert_eq!(
            third_party_attack.current_pad().dominance,
            threatened.current_pad().dominance
        );
    }

    #[test]
    fn direct_witness_rejects_unknown_observer_and_actor() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let unknown = MemberId(999);

        assert_eq!(
            simulation.submit_direct_witness(deed(30, unknown, actor, None, 0.1, 0.0, false)),
            Err(Error::MemberNotFound(unknown))
        );
        assert_eq!(
            simulation.submit_direct_witness(deed(31, observer, unknown, None, 0.1, 0.0, false)),
            Err(Error::MemberNotFound(unknown))
        );
        assert!(simulation.drain_events().next().is_none());
    }

    #[test]
    fn invalid_existing_pad_causes_atomic_failure() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        simulation.members.get_mut(&observer).unwrap().pad.pleasure = f32::NAN;
        simulation
            .set_member_relationship(observer, actor, affinity(0.2))
            .unwrap();

        assert_eq!(
            simulation.submit_direct_witness(deed(40, observer, actor, None, 0.5, 0.0, false)),
            Err(Error::Value(ValueError::NonFinite))
        );
        assert_eq!(
            simulation.member_affinity(observer, actor).unwrap(),
            affinity(0.0)
        );
        assert!(simulation.member_pad(observer).unwrap().pleasure.is_nan());
        let events = simulation.drain_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            SimulationEvent::RelationshipChanged { .. }
        ));
    }

    #[test]
    fn repeated_deeds_follow_a_deterministic_non_deduplicated_trajectory() {
        let (mut first, _, _, observer, actor) = relationship_simulation();
        let (mut second, _, _, second_observer, second_actor) = relationship_simulation();
        let first_deed = deed(50, observer, actor, None, 0.5, 0.2, false);
        let second_deed = deed(50, second_observer, second_actor, None, 0.5, 0.2, false);

        let first_once = first.submit_direct_witness(first_deed).unwrap();
        let first_twice = first.submit_direct_witness(first_deed).unwrap();
        let second_once = second.submit_direct_witness(second_deed).unwrap();
        assert_eq!(first_once, second_once);
        assert_eq!(
            first_twice,
            DirectWitnessSubmission::Duplicate {
                observer,
                deed_id: 50,
            }
        );
        assert_eq!(
            first.member_affinity(observer, actor).unwrap(),
            second
                .member_affinity(second_observer, second_actor)
                .unwrap()
        );
        assert_eq!(
            first.member_pad(observer),
            second.member_pad(second_observer)
        );
    }

    #[test]
    fn empty_batch_is_a_no_op() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let before_pad = simulation.member_pad(observer).unwrap();
        assert_eq!(
            simulation.submit_direct_witness_batch(&[]).unwrap(),
            Vec::new()
        );
        assert_eq!(
            simulation.member_affinity(observer, actor).unwrap(),
            affinity(0.0)
        );
        assert_eq!(simulation.member_pad(observer), Some(before_pad));
        assert!(simulation.drain_events().next().is_none());
    }

    #[test]
    fn batch_results_are_input_aligned_and_duplicate_items_are_explicit() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let deeds = [
            deed(60, observer, actor, None, 0.2, 0.0, false),
            deed(60, observer, actor, None, -1.0, 1.0, true),
            deed(61, observer, actor, None, 0.0, 0.0, false),
        ];
        let results = simulation.submit_direct_witness_batch(&deeds).unwrap();
        assert!(matches!(results[0], DirectWitnessSubmission::Applied(_)));
        assert_eq!(
            results[1],
            DirectWitnessSubmission::Duplicate {
                observer,
                deed_id: 60,
            }
        );
        assert!(matches!(results[2], DirectWitnessSubmission::Applied(_)));
        let events = simulation.drain_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 4);
        assert!(matches!(
            events[0],
            SimulationEvent::DeedEvaluated { deed_id: 60, .. }
        ));
        assert!(matches!(events[1], SimulationEvent::AffinityChanged { .. }));
        assert!(matches!(events[2], SimulationEvent::PadChanged { .. }));
        assert!(matches!(
            events[3],
            SimulationEvent::DeedEvaluated { deed_id: 61, .. }
        ));
    }

    #[test]
    fn batch_deduplication_is_per_observer_and_shared_with_single_submission() {
        let (mut simulation, faction_a, _, observer_a, actor) = relationship_simulation();
        let observer_b = simulation.add_member(faction_a).unwrap();
        simulation.drain_events().for_each(drop);

        let first = simulation
            .submit_direct_witness(deed(70, observer_a, actor, None, 0.4, 0.0, false))
            .unwrap();
        assert!(matches!(first, DirectWitnessSubmission::Applied(_)));
        let results = simulation
            .submit_direct_witness_batch(&[
                deed(70, observer_a, actor, None, 0.9, 0.0, false),
                deed(70, observer_b, actor, None, 0.4, 0.0, false),
            ])
            .unwrap();
        assert_eq!(
            results[0],
            DirectWitnessSubmission::Duplicate {
                observer: observer_a,
                deed_id: 70,
            }
        );
        assert!(matches!(results[1], DirectWitnessSubmission::Applied(_)));
        assert!(
            (simulation
                .member_affinity(observer_a, actor)
                .unwrap()
                .value()
                - 0.12)
                .abs()
                < 1e-6
        );
        assert!(
            simulation
                .member_affinity(observer_b, actor)
                .unwrap()
                .value()
                > 0.0
        );
    }

    #[test]
    fn batch_failure_rolls_back_prior_items_state_events_and_deduplication() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        simulation
            .set_member_relationship(observer, actor, affinity(0.5))
            .unwrap();
        simulation.drain_events().for_each(drop);
        let invalid = MemberId(999);
        let error = simulation
            .submit_direct_witness_batch(&[
                deed(80, observer, actor, None, 0.5, 0.0, false),
                deed(81, observer, actor, Some(invalid), 0.5, 0.0, false),
            ])
            .unwrap_err();
        assert_eq!(error.index(), 1);
        assert_eq!(error.error(), &Error::MemberNotFound(invalid));
        assert_eq!(
            simulation.member_affinity(observer, actor).unwrap(),
            affinity(0.0)
        );
        assert_eq!(simulation.member_pad(observer), Some(Pad::default()));
        assert!(simulation.drain_events().next().is_none());

        let retry = simulation
            .submit_direct_witness(deed(80, observer, actor, None, 0.5, 0.0, false))
            .unwrap();
        assert!(matches!(retry, DirectWitnessSubmission::Applied(_)));
    }

    #[test]
    fn batch_prevalidation_reports_first_invalid_index_without_side_effects() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let unknown = MemberId(404);
        let error = simulation
            .submit_direct_witness_batch(&[
                deed(90, observer, actor, None, 0.1, 0.0, false),
                deed(91, observer, unknown, None, 0.1, 0.0, false),
                deed(92, observer, actor, Some(actor), 0.1, 0.0, false),
            ])
            .unwrap_err();
        assert_eq!(error.index(), 1);
        assert_eq!(error.error(), &Error::MemberNotFound(unknown));
        assert_eq!(
            simulation.member_affinity(observer, actor).unwrap(),
            affinity(0.0)
        );
        assert!(simulation.drain_events().next().is_none());
    }

    #[test]
    fn batch_failure_after_duplicate_does_not_commit_duplicate_or_prior_application() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        simulation
            .submit_direct_witness(deed(100, observer, actor, None, 0.2, 0.0, false))
            .unwrap();
        simulation.drain_events().for_each(drop);
        let unknown = MemberId(505);
        let error = simulation
            .submit_direct_witness_batch(&[
                deed(100, observer, actor, None, 0.9, 0.0, false),
                deed(101, observer, actor, Some(unknown), 0.2, 0.0, false),
            ])
            .unwrap_err();
        assert_eq!(error.index(), 1);
        assert!((simulation.member_affinity(observer, actor).unwrap().value() - 0.06).abs() < 1e-6);
        assert!(simulation.drain_events().next().is_none());
        let fresh = simulation
            .submit_direct_witness(deed(101, observer, actor, None, 0.2, 0.0, false))
            .unwrap();
        assert!(matches!(fresh, DirectWitnessSubmission::Applied(_)));
    }

    #[test]
    fn memory_thresholds_and_record_fields_are_applied_to_outcome_and_queries() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let short = applied(
            simulation
                .submit_direct_witness(deed(200, observer, actor, None, 0.4, 0.0, false))
                .unwrap(),
        );
        let memory = short.memory().expect("threshold boundary is remembered");
        assert_eq!(memory.observer(), observer);
        assert_eq!(memory.deed_id(), 200);
        assert_eq!(memory.actor(), actor);
        assert_eq!(memory.target(), None);
        assert_eq!(memory.impact(), impact(0.4));
        assert_eq!(memory.aggression(), aggression(0.0));
        assert_eq!(memory.kind(), MemoryKind::ShortTerm);
        assert!((memory.salience() - 0.4).abs() < 1e-6);
        assert_eq!(simulation.memory(observer, 200).unwrap(), Some(memory));

        let long = applied(
            simulation
                .submit_direct_witness(deed(201, observer, actor, None, 0.0, 0.75, false))
                .unwrap(),
        );
        assert_eq!(long.memory().unwrap().kind(), MemoryKind::LongTerm);
        assert_eq!(simulation.memory(observer, 201).unwrap(), long.memory());

        let not_remembered = applied(
            simulation
                .submit_direct_witness(deed(202, observer, actor, None, 0.39, 0.0, false))
                .unwrap(),
        );
        assert_eq!(not_remembered.memory(), None);
        assert_eq!(simulation.memory(observer, 202).unwrap(), None);
    }

    #[test]
    fn aggression_only_can_create_memory_and_memory_events_follow_pad_events() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let outcome = applied(
            simulation
                .submit_direct_witness(deed(203, observer, actor, None, 0.0, 0.6, true))
                .unwrap(),
        );
        assert_eq!(outcome.memory().unwrap().kind(), MemoryKind::ShortTerm);
        let events = simulation.drain_events().collect::<Vec<_>>();
        assert!(matches!(events[0], SimulationEvent::DeedEvaluated { .. }));
        assert!(matches!(events[1], SimulationEvent::PadChanged { .. }));
        assert!(matches!(
            events[2],
            SimulationEvent::MemoryRemembered {
                kind: MemoryKind::ShortTerm,
                ..
            }
        ));
    }

    #[test]
    fn memory_queries_are_deterministic_and_reject_unknown_observers() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        simulation
            .submit_direct_witness_batch(&[
                deed(205, observer, actor, None, 0.5, 0.0, false),
                deed(204, observer, actor, None, 0.5, 0.0, false),
            ])
            .unwrap();
        let memories = simulation.memories_for(observer).unwrap();
        assert_eq!(
            memories
                .iter()
                .map(|memory| memory.deed_id())
                .collect::<Vec<_>>(),
            vec![204, 205]
        );
        assert_eq!(simulation.memory(observer, 999).unwrap(), None);
        let unknown = MemberId(9999);
        assert_eq!(
            simulation.memory(unknown, 1),
            Err(Error::MemberNotFound(unknown))
        );
        assert_eq!(
            simulation.memories_for(unknown),
            Err(Error::MemberNotFound(unknown))
        );
    }

    #[test]
    fn duplicate_deeds_do_not_change_or_reemit_memory() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        let first = applied(
            simulation
                .submit_direct_witness(deed(206, observer, actor, None, 0.5, 0.0, false))
                .unwrap(),
        );
        simulation.drain_events().for_each(drop);
        let duplicate = simulation
            .submit_direct_witness(deed(206, observer, actor, None, 1.0, 1.0, true))
            .unwrap();
        assert_eq!(
            duplicate,
            DirectWitnessSubmission::Duplicate {
                observer,
                deed_id: 206,
            }
        );
        assert_eq!(simulation.memory(observer, 206).unwrap(), first.memory());
        assert!(simulation.drain_events().next().is_none());
    }

    #[test]
    fn remembered_state_rolls_back_with_a_failed_batch() {
        let (mut simulation, _, _, observer, actor) = relationship_simulation();
        simulation.drain_events().for_each(drop);
        let unknown = MemberId(707);
        let error = simulation
            .submit_direct_witness_batch(&[
                deed(207, observer, actor, None, 0.5, 0.0, false),
                deed(208, observer, actor, Some(unknown), 0.5, 0.0, false),
            ])
            .unwrap_err();
        assert_eq!(error.index(), 1);
        assert_eq!(simulation.memory(observer, 207).unwrap(), None);
        assert!(simulation.memories_for(observer).unwrap().is_empty());
        assert!(simulation.drain_events().next().is_none());

        let retry = simulation
            .submit_direct_witness(deed(207, observer, actor, None, 0.5, 0.0, false))
            .unwrap();
        assert!(matches!(retry, DirectWitnessSubmission::Applied(_)));
        assert!(simulation.memory(observer, 207).unwrap().is_some());
    }
}
