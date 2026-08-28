use crate::values::{PadValue, ValueError};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_and_validation_cover_legacy_public_fields() {
        let pad = Pad::new(-0.0, 0.25, -1.0).unwrap();
        assert_eq!(pad.pleasure.to_bits(), 0.0f32.to_bits());
        assert_eq!(pad.validate(), Ok(()));

        let invalid = Pad {
            pleasure: 1.1,
            ..Pad::default()
        };
        assert_eq!(invalid.validate(), Err(ValueError::OutOfRange));
    }
}
