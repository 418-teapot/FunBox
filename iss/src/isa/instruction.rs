//! Typed instructions of the simulated core.

use super::rounding::RoundingMode;
use super::r#type::Type;

/// A decoded instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    /// Integer or floating-point addition.
    Add {
        /// Operand type of the addition.
        kind: Type,
        /// Ignored by integer types.
        rounding: RoundingMode,
        /// Whether the result saturates instead of wrapping.
        sat: bool,
        /// Destination register.
        rd: usize,
        /// First source register.
        rs1: usize,
        /// Second source register.
        rs2: usize,
    },
}

impl Instruction {
    pub(super) const fn name(&self) -> &'static str {
        match self {
            Self::Add { .. } => "add",
        }
    }
}
