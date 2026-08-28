/// Errors returned when constructing or updating normalized domain values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueError {
    /// The supplied value was NaN or either infinity.
    NonFinite,
    /// The supplied value was outside the type's documented range.
    OutOfRange,
}

pub(crate) fn validate_value(value: f32, min: f32, max: f32) -> Result<f32, ValueError> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
