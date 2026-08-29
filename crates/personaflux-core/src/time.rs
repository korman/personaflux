/// Errors raised while moving the simulation clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimeError {
    WentBackwards { current_tick: u64, target_tick: u64 },
}

/// Monotonic logical clock owned by one simulation instance.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LogicalClock {
    current_tick: u64,
}

impl LogicalClock {
    pub(crate) const fn new() -> Self {
        Self { current_tick: 0 }
    }

    pub(crate) const fn current_tick(self) -> u64 {
        self.current_tick
    }

    pub(crate) fn advance_to(&mut self, target_tick: u64) -> Result<(), TimeError> {
        if target_tick < self.current_tick {
            return Err(TimeError::WentBackwards {
                current_tick: self.current_tick,
                target_tick,
            });
        }
        self.current_tick = target_tick;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_starts_at_zero_and_is_monotonic() {
        let mut clock = LogicalClock::new();
        assert_eq!(clock.current_tick(), 0);
        clock.advance_to(3).unwrap();
        assert_eq!(clock.current_tick(), 3);
        clock.advance_to(9).unwrap();
        assert_eq!(clock.current_tick(), 9);
        assert_eq!(
            clock.advance_to(8),
            Err(TimeError::WentBackwards {
                current_tick: 9,
                target_tick: 8
            })
        );
        assert_eq!(clock.current_tick(), 9);
    }
}
