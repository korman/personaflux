use crate::values::{Affinity, PadValue, ValueError, validate_value};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn affinity(value: f32) -> Affinity {
        Affinity::new(value).unwrap()
    }

    #[test]
    fn scales_remaining_space_and_rejects_invalid_results() {
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
}
