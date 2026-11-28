//! Execution of decoded instructions.

use std::fmt;

use crate::isa::{Instruction, RoundingMode, Type};
use crate::regs::Regs;

/// Error returned when an instruction cannot be executed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecuteError {
    /// This build does not implement the type.
    UnsupportedType(Type),
    /// A floating-point instruction carries no rounding mode.
    InvalidRounding(Type),
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedType(kind) => {
                write!(formatter, "unsupported type {kind:?}")
            }
            Self::InvalidRounding(kind) => {
                write!(formatter, "invalid rounding mode for {kind:?}")
            }
        }
    }
}

impl std::error::Error for ExecuteError {}

/// Executes `instruction`, writing its result to `regs`.
///
/// Integer types ignore `rounding`.
///
/// # Panics
///
/// Panics if a register index is outside the register file; a decoded
/// instruction never carries one.
///
/// # Errors
///
/// Returns [`ExecuteError::UnsupportedType`] for a type that is not implemented
/// and [`ExecuteError::InvalidRounding`] for a floating-point instruction
/// without a rounding mode; `regs` is left unchanged.
pub fn execute(
    instruction: Instruction,
    regs: &mut Regs,
) -> Result<(), ExecuteError> {
    let Instruction::Add {
        kind,
        rounding,
        sat,
        rd,
        rs1,
        rs2,
    } = instruction;

    let lhs = regs.read(rs1);
    let rhs = regs.read(rs2);
    let value = match kind {
        Type::U64 => add_u64(lhs, rhs, sat),
        Type::I64 => add_i64(lhs, rhs, sat),
        _ => return Err(rejection(kind, rounding)),
    };

    regs.write(rd, value);

    Ok(())
}

fn rejection(kind: Type, rounding: RoundingMode) -> ExecuteError {
    if kind.is_float() && rounding == RoundingMode::None {
        ExecuteError::InvalidRounding(kind)
    } else {
        ExecuteError::UnsupportedType(kind)
    }
}

const fn add_u64(lhs: u64, rhs: u64, sat: bool) -> u64 {
    if sat {
        lhs.saturating_add(rhs)
    } else {
        lhs.wrapping_add(rhs)
    }
}

const fn add_i64(lhs: u64, rhs: u64, sat: bool) -> u64 {
    let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
    let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
    let sum = if sat {
        lhs.saturating_add(rhs)
    } else {
        lhs.wrapping_add(rhs)
    };

    u64::from_ne_bytes(sum.to_ne_bytes())
}

#[cfg(test)]
mod tests {
    use super::{ExecuteError, execute};
    use crate::isa::{Instruction, RoundingMode, Type};
    use crate::regs::Regs;

    fn add(
        kind: Type,
        sat: bool,
        rd: usize,
        rs1: usize,
        rs2: usize,
    ) -> Instruction {
        Instruction::Add {
            kind,
            rounding: RoundingMode::None,
            sat,
            rd,
            rs1,
            rs2,
        }
    }

    #[test]
    fn integer_addition_ignores_the_rounding_mode() {
        let roundings = [
            RoundingMode::None,
            RoundingMode::Rne,
            RoundingMode::Rtz,
            RoundingMode::Rdn,
            RoundingMode::Rup,
            RoundingMode::Rmm,
        ];
        let i64_bits = |value: i64| u64::from_ne_bytes(value.to_ne_bytes());
        let cases = [
            (Type::U64, u64::MAX, 1, 0_u64),
            (Type::I64, i64_bits(i64::MAX), 1, i64_bits(i64::MIN)),
        ];

        for (kind, lhs, rhs, expected) in cases {
            for rounding in roundings {
                let instruction = Instruction::Add {
                    kind,
                    rounding,
                    sat: false,
                    rd: 3,
                    rs1: 1,
                    rs2: 2,
                };
                let mut regs = Regs::RESET;

                regs.write(1, lhs);
                regs.write(2, rhs);

                let result = execute(instruction, &mut regs);

                assert_eq!(result, Ok(()), "{kind:?} {rounding:?} failed");
                assert_eq!(
                    regs.read(3),
                    expected,
                    "{kind:?} {rounding:?} gave the wrong result"
                );
            }
        }
    }

    #[test]
    fn rejected_instructions_leave_the_registers_unchanged() {
        let cases = [
            (
                Type::I32,
                RoundingMode::None,
                ExecuteError::UnsupportedType(Type::I32),
            ),
            (
                Type::U8,
                RoundingMode::Rne,
                ExecuteError::UnsupportedType(Type::U8),
            ),
            (
                Type::F32,
                RoundingMode::Rne,
                ExecuteError::UnsupportedType(Type::F32),
            ),
            (
                Type::F32,
                RoundingMode::None,
                ExecuteError::InvalidRounding(Type::F32),
            ),
        ];

        for (kind, rounding, expected) in cases {
            let mut regs = Regs::RESET;
            let mut reference = Regs::RESET;

            regs.write(1, 5);
            regs.write(2, 7);
            reference.write(1, 5);
            reference.write(2, 7);

            let instruction = Instruction::Add {
                kind,
                rounding,
                sat: false,
                rd: 3,
                rs1: 1,
                rs2: 2,
            };
            let result = execute(instruction, &mut regs);

            assert_eq!(
                result,
                Err(expected),
                "{kind:?} {rounding:?} was accepted"
            );
            assert_eq!(
                regs, reference,
                "{kind:?} {rounding:?} changed a register"
            );
        }
    }

    #[test]
    fn overlapping_operands_use_the_original_values() {
        let cases = [
            (1_usize, 1_usize, 2_usize, 16_u64),
            (2, 1, 2, 16),
            (1, 1, 1, 14),
        ];

        for (rd, rs1, rs2, expected) in cases {
            let mut regs = Regs::RESET;

            regs.write(1, 7);
            regs.write(2, 9);

            let result =
                execute(add(Type::U64, false, rd, rs1, rs2), &mut regs);

            assert_eq!(result, Ok(()), "add r{rd}, r{rs1}, r{rs2} failed");
            assert_eq!(
                regs.read(rd),
                expected,
                "add r{rd}, r{rs1}, r{rs2} gave the wrong result"
            );
        }
    }

    #[test]
    fn i64_addition_wraps_or_saturates() {
        let cases = [
            (5_i64, 7_i64, false, 12_i64),
            (-5, -7, true, -12),
            (i64::MAX, 1, false, i64::MIN),
            (i64::MAX, 1, true, i64::MAX),
            (i64::MIN, -1, false, i64::MAX),
            (i64::MIN, -1, true, i64::MIN),
            (i64::MIN, i64::MAX, false, -1),
        ];

        for (lhs, rhs, sat, expected) in cases {
            let mut regs = Regs::RESET;

            let lhs_bits = u64::from_ne_bytes(lhs.to_ne_bytes());
            let rhs_bits = u64::from_ne_bytes(rhs.to_ne_bytes());

            regs.write(1, lhs_bits);
            regs.write(2, rhs_bits);

            let result = execute(add(Type::I64, sat, 3, 1, 2), &mut regs);

            assert_eq!(result, Ok(()), "{lhs} + {rhs} (sat {sat}) failed");
            assert_eq!(
                regs.read(3),
                u64::from_ne_bytes(expected.to_ne_bytes()),
                "{lhs} + {rhs} (sat {sat}) gave the wrong result"
            );
        }
    }

    #[test]
    fn u64_addition_wraps_or_saturates() {
        let cases = [
            (2_u64, 3_u64, false, 5_u64),
            (u64::MAX, 1, false, 0),
            (u64::MAX, 1, true, u64::MAX),
            (u64::MAX - 1, u64::MAX, false, u64::MAX - 2),
        ];

        for (lhs, rhs, sat, expected) in cases {
            let mut regs = Regs::RESET;

            regs.write(1, lhs);
            regs.write(2, rhs);

            let result = execute(add(Type::U64, sat, 3, 1, 2), &mut regs);

            assert_eq!(
                result,
                Ok(()),
                "{lhs:#x} + {rhs:#x} (sat {sat}) failed"
            );
            assert_eq!(
                regs.read(3),
                expected,
                "{lhs:#x} + {rhs:#x} (sat {sat}) gave the wrong result"
            );
        }
    }
}
