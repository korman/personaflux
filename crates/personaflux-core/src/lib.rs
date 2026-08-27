#![forbid(unsafe_code)]

use std::collections::{BTreeMap, VecDeque};

/// Stable identifier for a faction inside one simulation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactionId(u64);

/// Stable identifier for a member inside one simulation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MemberId(u64);

/// Pleasure, arousal, and dominance state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pad {
    pub pleasure: f32,
    pub arousal: f32,
    pub dominance: f32,
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum SimulationEvent {
    FactionAdded { faction_id: FactionId },
    MemberAdded { member_id: MemberId },
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

    pub fn drain_events(&mut self) -> impl Iterator<Item = SimulationEvent> + '_ {
        self.events.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
