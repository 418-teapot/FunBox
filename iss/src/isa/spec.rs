//! Runtime model of the ISA table, shared by loading and codec.

use std::collections::BTreeMap;

use super::word::InstructionWord;

#[derive(Clone, Copy, Debug)]
pub(super) enum FieldKind {
    Fixed(u64),
    Type,
    Rounding,
    Register,
    Bool,
}

#[derive(Debug)]
pub(super) struct Field {
    pub(super) name: String,
    pub(super) lsb: u32,
    pub(super) width: u32,
    pub(super) kind: FieldKind,
}

#[derive(Debug)]
pub(super) struct InstructionSpec {
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
    pub(super) mask: u64,
    pub(super) pattern: u64,
}

/// The instruction set of the simulated core, read from a TOML table.
#[derive(Debug)]
pub struct Isa {
    pub(super) types: BTreeMap<String, u64>,
    pub(super) roundings: BTreeMap<String, u64>,
    pub(super) instructions: Vec<InstructionSpec>,
}

pub(super) const fn width_mask(width: u32) -> u64 {
    if width >= InstructionWord::BITS {
        u64::MAX
    } else {
        (1_u64 << width) - 1
    }
}

pub(super) const fn fits(value: u64, width: u32) -> bool {
    width >= InstructionWord::BITS || value >> width == 0
}
