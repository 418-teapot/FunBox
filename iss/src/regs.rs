//! Register file of the simulated core.

/// Number of registers in the file.
pub const COUNT: usize = 64;

const ZERO: usize = 0;

/// A file of [`COUNT`] registers; `r0` reads as `0` and ignores writes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Regs {
    slots: [u64; COUNT],
}

impl Default for Regs {
    fn default() -> Self {
        Self::new()
    }
}

impl Regs {
    /// Creates a register file in its reset state.
    #[must_use]
    pub const fn new() -> Self {
        Self { slots: [0; COUNT] }
    }

    /// # Panics
    ///
    /// Panics if `index` is out of range.
    #[must_use]
    pub const fn read(&self, index: usize) -> u64 {
        self.slots[index]
    }

    /// # Panics
    ///
    /// Panics if `index` is out of range.
    pub const fn write(&mut self, index: usize, value: u64) {
        if index != ZERO {
            self.slots[index] = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{COUNT, Regs};

    fn pattern(index: usize) -> u64 {
        let value = index as u64;

        value
            .wrapping_mul(0x0101_0101_0101_0101)
            .wrapping_add(0x00ff_00ff)
    }

    #[test]
    fn every_register_starts_at_zero() {
        let regs = Regs::new();

        for index in 0..COUNT {
            assert_eq!(regs.read(index), 0, "r{index} did not start at zero");
        }
    }

    #[test]
    fn writable_registers_round_trip_their_value() {
        let mut regs = Regs::new();

        for index in 1..COUNT {
            regs.write(index, pattern(index));
        }

        for index in 1..COUNT {
            assert_eq!(
                regs.read(index),
                pattern(index),
                "r{index} lost its value"
            );
        }
    }

    #[test]
    fn zero_register_ignores_writes() {
        let mut regs = Regs::new();

        regs.write(0, u64::MAX);
        regs.write(0, pattern(7));

        assert_eq!(regs.read(0), 0);
    }
}
