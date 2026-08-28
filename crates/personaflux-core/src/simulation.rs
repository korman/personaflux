use std::collections::{BTreeMap, VecDeque};

use crate::pad::Pad;
use crate::relationship::{
    RelationshipLayer, RelationshipLookup, RelationshipStore, RelationshipSubject,
};

/// Stable identifier for a faction inside one simulation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactionId(u64);

/// Stable identifier for a member inside one simulation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MemberId(u64);

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
    use crate::{Affinity, RelationshipLookup};

    fn affinity(value: f32) -> Affinity {
        Affinity::new(value).unwrap()
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
}
