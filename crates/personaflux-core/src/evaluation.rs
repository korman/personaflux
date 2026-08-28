use crate::pad::Pad;
use crate::values::{Affinity, Aggression, Confidence, Impact};

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
    fn policy_exposes_fixed_model_constants() {
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
}
