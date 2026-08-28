#![forbid(unsafe_code)]

use std::collections::{BTreeMap, VecDeque};

/// Errors returned when constructing or updating normalized domain values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueError {
    /// The supplied value was NaN or either infinity.
    NonFinite,
    /// The supplied value was outside the type's documented range.
    OutOfRange,
}

fn validate_value(value: f32, min: f32, max: f32) -> Result<f32, ValueError> {
    if !value.is_finite() {
        return Err(ValueError::NonFinite);
    }
    if value < min || value > max {
        return Err(ValueError::OutOfRange);
    }

    // The public numeric boundary normalizes negative zero.
    Ok(if value == 0.0 { 0.0 } else { value })
}

macro_rules! normalized_value {
    ($(#[$meta:meta])* $name:ident, $min:expr, $max:expr) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(f32);

        impl $name {
            /// The smallest valid value.
            pub const MIN: f32 = $min;
            /// The largest valid value.
            pub const MAX: f32 = $max;

            /// Constructs a value after checking finiteness and range.
            pub fn new(value: f32) -> Result<Self, ValueError> {
                Ok(Self(validate_value(value, Self::MIN, Self::MAX)?))
            }

            /// Returns the normalized binary32 value.
            pub fn value(self) -> f32 {
                self.0
            }

            /// Alias for [`Self::value`].
            pub fn as_f32(self) -> f32 {
                self.value()
            }
        }

        impl TryFrom<f32> for $name {
            type Error = ValueError;

            fn try_from(value: f32) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

normalized_value!(
    /// Directional affinity toward another entity.
    Affinity,
    -1.0,
    1.0
);

normalized_value!(
    /// Signed beneficial or harmful result of a deed.
    Impact,
    -1.0,
    1.0
);

normalized_value!(
    /// Non-negative aggression or threat intensity of a deed.
    Aggression,
    0.0,
    1.0
);

normalized_value!(
    /// Confidence in an observed fact.
    Confidence,
    0.0,
    1.0
);

normalized_value!(
    /// One normalized PAD axis value.
    PadValue,
    -1.0,
    1.0
);

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

impl Pad {
    /// Constructs a PAD value after validating and normalizing all axes.
    pub fn new(pleasure: f32, arousal: f32, dominance: f32) -> Result<Self, ValueError> {
        Ok(Self {
            pleasure: PadValue::new(pleasure)?.value(),
            arousal: PadValue::new(arousal)?.value(),
            dominance: PadValue::new(dominance)?.value(),
        })
    }

    /// Validates a PAD value, including values written through its legacy public fields.
    pub fn validate(&self) -> Result<(), ValueError> {
        PadValue::new(self.pleasure)?;
        PadValue::new(self.arousal)?;
        PadValue::new(self.dominance)?;
        Ok(())
    }
}

/// Immutable configuration for the direct-witness v1 evaluation model.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EvaluationPolicyV1;

impl EvaluationPolicyV1 {
    pub const MODEL_VERSION: u32 = 1;
    pub const MORAL_BASELINE: f32 = 0.30;
    pub const RELATIONSHIP_WEIGHT: f32 = 0.70;
    pub const MEMORY_THRESHOLD: f32 = 0.40;
    pub const LONG_TERM_THRESHOLD: f32 = 0.75;

    pub const fn new() -> Self {
        Self
    }

    pub const fn model_version(self) -> u32 {
        Self::MODEL_VERSION
    }

    pub const fn moral_baseline(self) -> f32 {
        Self::MORAL_BASELINE
    }

    pub const fn relationship_weight(self) -> f32 {
        Self::RELATIONSHIP_WEIGHT
    }

    pub const fn memory_threshold(self) -> f32 {
        Self::MEMORY_THRESHOLD
    }

    pub const fn long_term_threshold(self) -> f32 {
        Self::LONG_TERM_THRESHOLD
    }

    /// Evaluates one direct witness without changing simulation state.
    pub fn evaluate_direct_witness(self, input: DirectWitnessInput) -> EvaluationResult {
        evaluate_with_policy(self, input)
    }
}

/// Inputs supplied by a host for one direct-witness evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectWitnessInput {
    pub impact: Impact,
    pub aggression: Aggression,
    pub target_affinity: Affinity,
    pub has_target: bool,
    pub threatens_observer: bool,
}

impl DirectWitnessInput {
    pub const fn new(
        impact: Impact,
        aggression: Aggression,
        target_affinity: Affinity,
        has_target: bool,
        threatens_observer: bool,
    ) -> Self {
        Self {
            impact,
            aggression,
            target_affinity,
            has_target,
            threatens_observer,
        }
    }
}

/// Classification assigned by the v1 memory salience thresholds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryDecision {
    None,
    ShortTerm,
    LongTerm,
}

/// Alias matching the domain term used by the behavior specification.
pub type MemoryClassification = MemoryDecision;

/// Pure output of one direct-witness evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EvaluationResult {
    concern: f32,
    effective_confidence: Confidence,
    raw_affinity_delta: f32,
    raw_pad_delta: Pad,
    event_intensity: f32,
    memory_salience: f32,
    memory_decision: MemoryDecision,
}

impl EvaluationResult {
    pub fn concern(self) -> f32 {
        self.concern
    }

    pub fn effective_confidence(self) -> Confidence {
        self.effective_confidence
    }

    pub fn raw_affinity_delta(self) -> f32 {
        self.raw_affinity_delta
    }

    pub fn raw_pad_delta(self) -> Pad {
        self.raw_pad_delta
    }

    pub fn event_intensity(self) -> f32 {
        self.event_intensity
    }

    pub fn memory_salience(self) -> f32 {
        self.memory_salience
    }

    pub fn memory_decision(self) -> MemoryDecision {
        self.memory_decision
    }
}

/// Evaluates a direct witness with the fixed v1 policy.
pub fn evaluate_direct_witness(input: DirectWitnessInput) -> EvaluationResult {
    EvaluationPolicyV1::new().evaluate_direct_witness(input)
}

fn evaluate_with_policy(policy: EvaluationPolicyV1, input: DirectWitnessInput) -> EvaluationResult {
    let effective_confidence = Confidence::new(1.0).expect("v1 confidence constant is valid");
    let target_affinity = if input.has_target {
        input.target_affinity.value()
    } else {
        0.0
    };
    let threatens_observer = input.has_target && input.threatens_observer;
    let concern = policy.moral_baseline() + policy.relationship_weight() * target_affinity;
    let raw_affinity_delta = input.impact.value() * concern;
    let event_intensity = input.impact.value().abs().max(input.aggression.value());
    let raw_pad_delta = Pad {
        pleasure: 0.5 * raw_affinity_delta,
        arousal: 0.5 * event_intensity,
        dominance: if threatens_observer {
            -0.4 * input.aggression.value()
        } else {
            0.0
        },
    };
    let memory_salience = event_intensity;
    let memory_decision = if memory_salience >= policy.long_term_threshold() {
        MemoryDecision::LongTerm
    } else if memory_salience >= policy.memory_threshold() {
        MemoryDecision::ShortTerm
    } else {
        MemoryDecision::None
    };

    EvaluationResult {
        concern,
        effective_confidence,
        raw_affinity_delta,
        raw_pad_delta,
        event_intensity,
        memory_salience,
        memory_decision,
    }
}

/// Applies a finite normalized delta while preserving the `[-1, 1]` boundary.
pub fn apply_bounded_delta(current_value: f32, raw_delta: f32) -> Result<f32, ValueError> {
    let current_value = validate_value(current_value, -1.0, 1.0)?;
    if !raw_delta.is_finite() {
        return Err(ValueError::NonFinite);
    }

    let applied_delta = if raw_delta >= 0.0 {
        raw_delta * (1.0 - current_value)
    } else {
        raw_delta * (1.0 + current_value)
    };
    let new_value = current_value + applied_delta;
    validate_value(new_value, -1.0, 1.0)
}

/// Applies a bounded delta to an affinity value.
pub fn apply_affinity_delta(
    current_value: Affinity,
    raw_delta: f32,
) -> Result<Affinity, ValueError> {
    Affinity::new(apply_bounded_delta(current_value.value(), raw_delta)?)
}

/// Applies a bounded delta to one PAD axis value.
pub fn apply_pad_value_delta(
    current_value: PadValue,
    raw_delta: f32,
) -> Result<PadValue, ValueError> {
    PadValue::new(apply_bounded_delta(current_value.value(), raw_delta)?)
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

    fn affinity(value: f32) -> Affinity {
        Affinity::new(value).unwrap()
    }

    fn impact(value: f32) -> Impact {
        Impact::new(value).unwrap()
    }

    fn aggression(value: f32) -> Aggression {
        Aggression::new(value).unwrap()
    }

    #[test]
    fn normalized_values_validate_ranges_and_normalize_negative_zero() {
        assert_eq!(Affinity::new(-1.0).unwrap().value(), -1.0);
        assert_eq!(Impact::new(1.0).unwrap().value(), 1.0);
        assert_eq!(Aggression::new(0.0).unwrap().value(), 0.0);
        assert_eq!(Confidence::new(1.0).unwrap().value(), 1.0);
        assert_eq!(
            PadValue::new(-0.0).unwrap().value().to_bits(),
            0.0f32.to_bits()
        );

        assert_eq!(Affinity::new(-1.000_000_1), Err(ValueError::OutOfRange));
        assert_eq!(Affinity::new(1.000_000_1), Err(ValueError::OutOfRange));
        assert_eq!(Impact::new(-1.000_000_1), Err(ValueError::OutOfRange));
        assert_eq!(Impact::new(1.000_000_1), Err(ValueError::OutOfRange));
        assert_eq!(Aggression::new(-f32::EPSILON), Err(ValueError::OutOfRange));
        assert_eq!(Aggression::new(1.000_000_1), Err(ValueError::OutOfRange));
        assert_eq!(Confidence::new(-f32::EPSILON), Err(ValueError::OutOfRange));
        assert_eq!(Confidence::new(1.000_000_1), Err(ValueError::OutOfRange));
        assert_eq!(Confidence::new(f32::NAN), Err(ValueError::NonFinite));
        assert_eq!(Impact::new(f32::INFINITY), Err(ValueError::NonFinite));
        assert_eq!(PadValue::new(f32::NEG_INFINITY), Err(ValueError::NonFinite));
    }

    #[test]
    fn pad_constructor_and_validation_cover_legacy_public_fields() {
        let pad = Pad::new(-0.0, 0.25, -1.0).unwrap();
        assert_eq!(pad.pleasure.to_bits(), 0.0f32.to_bits());
        assert_eq!(pad.validate(), Ok(()));

        let invalid = Pad {
            pleasure: 1.1,
            ..Pad::default()
        };
        assert_eq!(invalid.validate(), Err(ValueError::OutOfRange));
    }

    #[test]
    fn v1_policy_exposes_fixed_model_constants() {
        let policy = EvaluationPolicyV1::new();
        assert_eq!(policy.model_version(), 1);
        assert_eq!(policy.moral_baseline(), 0.30);
        assert_eq!(policy.relationship_weight(), 0.70);
        assert_eq!(policy.memory_threshold(), 0.40);
        assert_eq!(policy.long_term_threshold(), 0.75);
    }

    #[test]
    fn direct_witness_uses_signed_concern_and_long_term_threshold() {
        let result = evaluate_direct_witness(DirectWitnessInput::new(
            impact(0.8),
            aggression(0.2),
            affinity(0.5),
            true,
            false,
        ));

        assert!((result.concern() - 0.65).abs() < 1e-6);
        assert!((result.raw_affinity_delta() - 0.52).abs() < 1e-6);
        assert_eq!(result.effective_confidence().value(), 1.0);
        assert!((result.event_intensity() - 0.8).abs() < 1e-6);
        assert!((result.raw_pad_delta().pleasure - 0.26).abs() < 1e-6);
        assert!((result.raw_pad_delta().arousal - 0.4).abs() < 1e-6);
        assert_eq!(result.raw_pad_delta().dominance, 0.0);
        assert_eq!(result.memory_decision(), MemoryDecision::LongTerm);
    }

    #[test]
    fn affinity_direction_covers_liked_neutral_and_disliked_targets() {
        let liked_harmed = evaluate_direct_witness(DirectWitnessInput::new(
            impact(-0.5),
            aggression(0.0),
            affinity(1.0),
            true,
            false,
        ));
        let neutral_harmed = evaluate_direct_witness(DirectWitnessInput::new(
            impact(-0.5),
            aggression(0.0),
            affinity(0.0),
            false,
            false,
        ));
        let disliked_harmed = evaluate_direct_witness(DirectWitnessInput::new(
            impact(-0.5),
            aggression(0.0),
            affinity(-1.0),
            true,
            false,
        ));

        assert!(liked_harmed.raw_affinity_delta() < 0.0);
        assert!((neutral_harmed.concern() - 0.30).abs() < 1e-6);
        assert!(neutral_harmed.raw_affinity_delta() < 0.0);
        assert!(disliked_harmed.raw_affinity_delta() > 0.0);

        let liked_helped = evaluate_direct_witness(DirectWitnessInput::new(
            impact(0.5),
            aggression(0.0),
            affinity(1.0),
            true,
            false,
        ));
        assert!(liked_helped.raw_affinity_delta() > 0.0);
    }

    #[test]
    fn no_target_ignores_relationship_and_observer_threat_flag() {
        let result = evaluate_direct_witness(DirectWitnessInput::new(
            impact(0.2),
            aggression(0.6),
            affinity(-1.0),
            false,
            true,
        ));

        assert!((result.concern() - 0.30).abs() < 1e-6);
        assert!((result.raw_affinity_delta() - 0.06).abs() < 1e-6);
        assert!((result.raw_pad_delta().arousal - 0.3).abs() < 1e-6);
        assert_eq!(result.raw_pad_delta().dominance, 0.0);
        assert_eq!(result.memory_decision(), MemoryDecision::ShortTerm);
    }

    #[test]
    fn aggression_can_raise_arousal_without_affinity_change() {
        let result = evaluate_direct_witness(DirectWitnessInput::new(
            impact(0.0),
            aggression(0.6),
            affinity(0.0),
            true,
            true,
        ));

        assert_eq!(result.raw_affinity_delta(), 0.0);
        assert!((result.raw_pad_delta().arousal - 0.3).abs() < 1e-6);
        assert!((result.raw_pad_delta().dominance + 0.24).abs() < 1e-6);
        assert!((result.memory_salience() - 0.6).abs() < 1e-6);
        assert_eq!(result.memory_decision(), MemoryDecision::ShortTerm);
    }

    #[test]
    fn memory_thresholds_include_their_boundary_values() {
        let short_term = evaluate_direct_witness(DirectWitnessInput::new(
            impact(0.4),
            aggression(0.0),
            affinity(0.0),
            true,
            false,
        ));
        let long_term = evaluate_direct_witness(DirectWitnessInput::new(
            impact(0.75),
            aggression(0.0),
            affinity(0.0),
            true,
            false,
        ));
        let none = evaluate_direct_witness(DirectWitnessInput::new(
            impact(0.0),
            aggression(0.0),
            affinity(0.0),
            true,
            false,
        ));

        assert_eq!(short_term.memory_decision(), MemoryDecision::ShortTerm);
        assert_eq!(long_term.memory_decision(), MemoryDecision::LongTerm);
        assert_eq!(none.memory_decision(), MemoryDecision::None);
    }

    #[test]
    fn bounded_delta_scales_remaining_space_and_rejects_invalid_results() {
        assert!((apply_bounded_delta(0.8, 0.5).unwrap() - 0.9).abs() < 1e-6);
        assert!((apply_bounded_delta(-0.8, -0.5).unwrap() + 0.9).abs() < 1e-6);
        assert_eq!(apply_bounded_delta(1.0, 1.0).unwrap(), 1.0);
        assert_eq!(apply_bounded_delta(-1.0, -1.0).unwrap(), -1.0);
        assert_eq!(apply_bounded_delta(0.0, 2.0), Err(ValueError::OutOfRange));
        assert_eq!(apply_bounded_delta(2.0, 0.0), Err(ValueError::OutOfRange));
        assert_eq!(
            apply_bounded_delta(0.0, f32::NAN),
            Err(ValueError::NonFinite)
        );
        assert_eq!(
            apply_bounded_delta(0.0, f32::INFINITY),
            Err(ValueError::NonFinite)
        );

        let current_affinity = affinity(0.8);
        assert!((apply_affinity_delta(current_affinity, 0.5).unwrap().value() - 0.9).abs() < 1e-6);
        let current_pad = PadValue::new(-0.8).unwrap();
        assert!((apply_pad_value_delta(current_pad, -0.5).unwrap().value() + 0.9).abs() < 1e-6);
    }

    #[test]
    fn evaluation_is_deterministic_and_does_not_mutate_input() {
        let input =
            DirectWitnessInput::new(impact(-0.35), aggression(0.2), affinity(0.25), true, false);
        let first = evaluate_direct_witness(input);
        let second = evaluate_direct_witness(input);
        assert_eq!(first, second);
        assert_eq!(input.impact.value(), -0.35);
        assert_eq!(input.aggression.value(), 0.2);
        assert_eq!(input.target_affinity.value(), 0.25);
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
}
