use std::collections::BTreeMap;

use crate::deed::DirectWitnessDeed;
use crate::evaluation::{EvaluationResult, MemoryDecision};
use crate::simulation::{Error, MemberId};
use crate::values::{Aggression, Impact};

pub(crate) const SHORT_TERM_TTL: u64 = 60;
pub(crate) const LONG_TERM_TTL: u64 = 3600;

/// The v1 retention class assigned to a remembered deed.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryKind {
    ShortTerm,
    LongTerm,
}

/// A read-only record of a deed remembered by one observer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MemoryRecord {
    observer: MemberId,
    deed_id: u64,
    actor: MemberId,
    target: Option<MemberId>,
    impact: Impact,
    aggression: Aggression,
    salience: f32,
    kind: MemoryKind,
    created_tick: u64,
    expires_at: u64,
}

impl MemoryRecord {
    pub(crate) fn from_evaluation(
        deed: DirectWitnessDeed,
        evaluation: EvaluationResult,
        created_tick: u64,
    ) -> Result<Option<Self>, Error> {
        let kind = match evaluation.memory_decision() {
            MemoryDecision::None => return Ok(None),
            MemoryDecision::ShortTerm => MemoryKind::ShortTerm,
            MemoryDecision::LongTerm => MemoryKind::LongTerm,
        };
        let expires_at = created_tick
            .checked_add(Self::ttl(kind))
            .ok_or(Error::TimeOverflow)?;

        Ok(Some(Self {
            observer: deed.observer,
            deed_id: deed.deed_id,
            actor: deed.actor,
            target: deed.target,
            impact: deed.impact,
            aggression: deed.aggression,
            salience: evaluation.memory_salience(),
            kind,
            created_tick,
            expires_at,
        }))
    }

    pub(crate) const fn ttl(kind: MemoryKind) -> u64 {
        match kind {
            MemoryKind::ShortTerm => SHORT_TERM_TTL,
            MemoryKind::LongTerm => LONG_TERM_TTL,
        }
    }

    pub const fn observer(self) -> MemberId {
        self.observer
    }

    pub const fn deed_id(self) -> u64 {
        self.deed_id
    }

    pub const fn actor(self) -> MemberId {
        self.actor
    }

    pub const fn target(self) -> Option<MemberId> {
        self.target
    }

    pub const fn impact(self) -> Impact {
        self.impact
    }

    pub const fn aggression(self) -> Aggression {
        self.aggression
    }

    pub const fn salience(self) -> f32 {
        self.salience
    }

    pub const fn kind(self) -> MemoryKind {
        self.kind
    }

    pub const fn created_tick(self) -> u64 {
        self.created_tick
    }

    pub const fn expires_at(self) -> u64 {
        self.expires_at
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum MemoryChange {
    None,
    Remembered(MemoryRecord),
    Upgraded {
        previous: MemoryKind,
        current: MemoryRecord,
    },
}

/// Deterministic in-memory storage for remembered deeds.
#[derive(Clone, Debug, Default)]
pub(crate) struct MemoryStore {
    records: BTreeMap<(MemberId, u64), MemoryRecord>,
}

impl MemoryStore {
    pub(crate) fn get(&self, observer: MemberId, deed_id: u64) -> Option<MemoryRecord> {
        self.records.get(&(observer, deed_id)).copied()
    }

    pub(crate) fn for_observer(&self, observer: MemberId) -> Vec<MemoryRecord> {
        self.records
            .iter()
            .filter_map(|(&(record_observer, _), &record)| {
                (record_observer == observer).then_some(record)
            })
            .collect()
    }

    pub(crate) fn insert_or_upgrade(
        &mut self,
        incoming: MemoryRecord,
        current_tick: u64,
    ) -> Result<MemoryChange, Error> {
        let key = (incoming.observer, incoming.deed_id);
        let Some(existing) = self.records.get(&key).copied() else {
            self.records.insert(key, incoming);
            return Ok(MemoryChange::Remembered(incoming));
        };

        if existing.kind == MemoryKind::ShortTerm && incoming.kind == MemoryKind::LongTerm {
            // The first deed payload remains authoritative; only retention metadata upgrades.
            let expires_at = current_tick
                .checked_add(LONG_TERM_TTL)
                .ok_or(Error::TimeOverflow)?;
            let current = MemoryRecord {
                salience: existing.salience.max(incoming.salience),
                kind: MemoryKind::LongTerm,
                expires_at,
                ..existing
            };
            self.records.insert(key, current);
            return Ok(MemoryChange::Upgraded {
                previous: existing.kind,
                current,
            });
        }

        Ok(MemoryChange::None)
    }

    pub(crate) fn expire(&mut self, current_tick: u64) -> Vec<MemoryRecord> {
        let expired_keys = self
            .records
            .iter()
            .filter_map(|(&key, record)| (current_tick >= record.expires_at()).then_some(key))
            .collect::<Vec<_>>();
        expired_keys
            .into_iter()
            .filter_map(|key| self.records.remove(&key))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deed(deed_id: u64, impact: f32, aggression: f32) -> DirectWitnessDeed {
        DirectWitnessDeed::new(
            deed_id,
            MemberId::from_raw(1),
            MemberId::from_raw(2),
            None,
            Impact::new(impact).unwrap(),
            Aggression::new(aggression).unwrap(),
            false,
        )
    }

    fn evaluation(impact: f32, aggression: f32, decision: MemoryDecision) -> EvaluationResult {
        let evaluation =
            crate::evaluation::evaluate_direct_witness(crate::evaluation::DirectWitnessInput::new(
                Impact::new(impact).unwrap(),
                Aggression::new(aggression).unwrap(),
                crate::Affinity::new(0.0).unwrap(),
                false,
                false,
            ));
        assert_eq!(evaluation.memory_decision(), decision);
        evaluation
    }

    #[test]
    fn remembers_once_and_orders_records_by_deed_id() {
        let mut store = MemoryStore::default();
        let first = MemoryRecord::from_evaluation(
            deed(2, 0.5, 0.0),
            evaluation(0.5, 0.0, MemoryDecision::ShortTerm),
            0,
        )
        .unwrap()
        .unwrap();
        let second = MemoryRecord::from_evaluation(
            deed(1, 0.8, 0.0),
            evaluation(0.8, 0.0, MemoryDecision::LongTerm),
            0,
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            store.insert_or_upgrade(first, 0).unwrap(),
            MemoryChange::Remembered(first)
        );
        assert_eq!(
            store.insert_or_upgrade(second, 0).unwrap(),
            MemoryChange::Remembered(second)
        );
        assert_eq!(
            store.for_observer(MemberId::from_raw(1)),
            vec![second, first]
        );
    }

    #[test]
    fn upgrades_short_term_without_replacing_the_original_payload() {
        let mut store = MemoryStore::default();
        let short = MemoryRecord::from_evaluation(
            deed(3, 0.5, 0.0),
            evaluation(0.5, 0.0, MemoryDecision::ShortTerm),
            4,
        )
        .unwrap()
        .unwrap();
        let long = MemoryRecord::from_evaluation(
            deed(3, -0.9, 0.8),
            evaluation(0.9, 0.8, MemoryDecision::LongTerm),
            4,
        )
        .unwrap()
        .unwrap();

        store.insert_or_upgrade(short, 4).unwrap();
        let current = match store.insert_or_upgrade(long, 10).unwrap() {
            MemoryChange::Upgraded { current, .. } => current,
            other => panic!("unexpected change: {other:?}"),
        };
        assert_eq!(current.actor(), short.actor());
        assert_eq!(current.impact(), short.impact());
        assert_eq!(current.kind(), MemoryKind::LongTerm);
        assert_eq!(current.salience(), long.salience());
        assert_eq!(current.created_tick(), short.created_tick());
        assert_eq!(current.expires_at(), 10 + LONG_TERM_TTL);
    }

    #[test]
    fn same_kind_and_downgrade_requests_are_no_ops() {
        let mut store = MemoryStore::default();
        let long = MemoryRecord::from_evaluation(
            deed(4, 0.9, 0.0),
            evaluation(0.9, 0.0, MemoryDecision::LongTerm),
            0,
        )
        .unwrap()
        .unwrap();
        let short = MemoryRecord::from_evaluation(
            deed(4, 0.4, 0.0),
            evaluation(0.4, 0.0, MemoryDecision::ShortTerm),
            0,
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            store.insert_or_upgrade(long, 0).unwrap(),
            MemoryChange::Remembered(long)
        );
        assert_eq!(
            store.insert_or_upgrade(long, 0).unwrap(),
            MemoryChange::None
        );
        assert_eq!(
            store.insert_or_upgrade(short, 0).unwrap(),
            MemoryChange::None
        );
        assert_eq!(store.get(MemberId::from_raw(1), 4), Some(long));
    }

    #[test]
    fn expiration_is_inclusive_and_keeps_records_before_the_boundary() {
        let mut store = MemoryStore::default();
        let record = MemoryRecord::from_evaluation(
            deed(5, 0.4, 0.0),
            evaluation(0.4, 0.0, MemoryDecision::ShortTerm),
            0,
        )
        .unwrap()
        .unwrap();
        store.insert_or_upgrade(record, 0).unwrap();
        assert!(store.expire(record.expires_at() - 1).is_empty());
        assert_eq!(store.expire(record.expires_at()), vec![record]);
        assert!(store.get(record.observer(), record.deed_id()).is_none());
    }
}
