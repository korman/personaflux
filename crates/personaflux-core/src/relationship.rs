use std::collections::BTreeMap;

use crate::values::Affinity;

/// Relationship storage layer used to explain an effective relationship lookup.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RelationshipLayer {
    MemberToMember,
    FactionToMember,
    FactionToFaction,
}

/// A typed relationship endpoint pair for change events.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RelationshipSubject {
    MemberToMember {
        observer: super::MemberId,
        target: super::MemberId,
    },
    FactionToMember {
        faction: super::FactionId,
        member: super::MemberId,
    },
    FactionToFaction {
        source: super::FactionId,
        target: super::FactionId,
    },
}

/// Result of looking up a relationship, preserving missing versus explicit neutral.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RelationshipLookup {
    Missing,
    Explicit {
        affinity: Affinity,
        source: RelationshipLayer,
    },
}

impl RelationshipLookup {
    pub fn explicit(affinity: Affinity, source: RelationshipLayer) -> Self {
        Self::Explicit { affinity, source }
    }

    pub fn affinity(self) -> Option<Affinity> {
        match self {
            Self::Missing => None,
            Self::Explicit { affinity, .. } => Some(affinity),
        }
    }

    pub fn source(self) -> Option<RelationshipLayer> {
        match self {
            Self::Missing => None,
            Self::Explicit { source, .. } => Some(source),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RelationshipStore {
    member_to_member: BTreeMap<(super::MemberId, super::MemberId), Affinity>,
    faction_to_member: BTreeMap<(super::FactionId, super::MemberId), Affinity>,
    faction_to_faction: BTreeMap<(super::FactionId, super::FactionId), Affinity>,
}

impl RelationshipStore {
    pub(crate) fn member_to_member(
        &self,
        observer: super::MemberId,
        target: super::MemberId,
    ) -> RelationshipLookup {
        self.member_to_member
            .get(&(observer, target))
            .copied()
            .map_or(RelationshipLookup::Missing, |affinity| {
                RelationshipLookup::explicit(affinity, RelationshipLayer::MemberToMember)
            })
    }

    pub(crate) fn faction_to_member(
        &self,
        faction: super::FactionId,
        member: super::MemberId,
    ) -> RelationshipLookup {
        self.faction_to_member
            .get(&(faction, member))
            .copied()
            .map_or(RelationshipLookup::Missing, |affinity| {
                RelationshipLookup::explicit(affinity, RelationshipLayer::FactionToMember)
            })
    }

    pub(crate) fn faction_to_faction(
        &self,
        source: super::FactionId,
        target: super::FactionId,
    ) -> RelationshipLookup {
        self.faction_to_faction
            .get(&(source, target))
            .copied()
            .map_or(RelationshipLookup::Missing, |affinity| {
                RelationshipLookup::explicit(affinity, RelationshipLayer::FactionToFaction)
            })
    }

    pub(crate) fn set_member_to_member(
        &mut self,
        observer: super::MemberId,
        target: super::MemberId,
        affinity: Affinity,
    ) -> Option<Affinity> {
        self.member_to_member.insert((observer, target), affinity)
    }

    pub(crate) fn clear_member_to_member(
        &mut self,
        observer: super::MemberId,
        target: super::MemberId,
    ) -> Option<Affinity> {
        self.member_to_member.remove(&(observer, target))
    }

    pub(crate) fn set_faction_to_member(
        &mut self,
        faction: super::FactionId,
        member: super::MemberId,
        affinity: Affinity,
    ) -> Option<Affinity> {
        self.faction_to_member.insert((faction, member), affinity)
    }

    pub(crate) fn clear_faction_to_member(
        &mut self,
        faction: super::FactionId,
        member: super::MemberId,
    ) -> Option<Affinity> {
        self.faction_to_member.remove(&(faction, member))
    }

    pub(crate) fn set_faction_to_faction(
        &mut self,
        source: super::FactionId,
        target: super::FactionId,
        affinity: Affinity,
    ) -> Option<Affinity> {
        self.faction_to_faction.insert((source, target), affinity)
    }

    pub(crate) fn clear_faction_to_faction(
        &mut self,
        source: super::FactionId,
        target: super::FactionId,
    ) -> Option<Affinity> {
        self.faction_to_faction.remove(&(source, target))
    }
}
