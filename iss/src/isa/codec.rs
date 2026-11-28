//! Encoding and decoding between instructions and words.

use std::collections::BTreeMap;
use std::fmt;

use super::instruction::Instruction;
use super::rounding::RoundingMode;
use super::spec::{Field, FieldKind, InstructionSpec, Isa, fits, width_mask};
use super::r#type::Type;
use super::word::InstructionWord;

/// Error returned when a word or an instruction does not map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The word matches no instruction of the table.
    InvalidEncoding,
    /// The table has the instruction, but this build has no variant for it.
    UnsupportedInstruction(String),
    /// The table has the instruction, but this build cannot encode its
    /// operand.
    UnsupportedOperand(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEncoding => {
                write!(formatter, "not an instruction of the ISA table")
            }
            Self::UnsupportedInstruction(name) => {
                write!(formatter, "unsupported instruction {name:?}")
            }
            Self::UnsupportedOperand(name) => {
                write!(formatter, "unsupported operand {name:?}")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Operand value at the encoding boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operand {
    Type(Type),
    Rounding(RoundingMode),
    Register(usize),
    Bool(bool),
}

impl Operand {
    const fn type_of(self) -> Option<Type> {
        match self {
            Self::Type(kind) => Some(kind),
            _ => None,
        }
    }

    const fn rounding_of(self) -> Option<RoundingMode> {
        match self {
            Self::Rounding(mode) => Some(mode),
            _ => None,
        }
    }

    const fn register_of(self) -> Option<usize> {
        match self {
            Self::Register(index) => Some(index),
            _ => None,
        }
    }

    const fn bool_of(self) -> Option<bool> {
        match self {
            Self::Bool(flag) => Some(flag),
            _ => None,
        }
    }
}

impl Isa {
    /// Returns `None` when the table has no encoding for the instruction, or
    /// when one of its operands does not fit the corresponding field.
    #[must_use]
    pub fn encode(&self, instruction: &Instruction) -> Option<InstructionWord> {
        let spec = self
            .instructions
            .iter()
            .find(|spec| spec.name == instruction.name())?;
        let mut fields: Vec<(&str, Operand)> = Vec::new();

        visit(instruction, |name, operand| fields.push((name, operand)));

        self.encode_fields(spec, &fields)
            .map(InstructionWord::from_bits)
    }

    /// Returns `None` when [`Isa::encode`] does.
    #[must_use]
    pub fn encode_bytes(
        &self,
        instruction: &Instruction,
    ) -> Option<[u8; InstructionWord::BYTES]> {
        self.encode(instruction).map(InstructionWord::to_le_bytes)
    }

    /// Decodes `word` into an instruction of the table.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError::InvalidEncoding`] when the word is not an
    /// instruction of the table. Returns an unsupported error when the table
    /// names an instruction or an operand that this build lacks.
    pub fn decode(
        &self,
        word: InstructionWord,
    ) -> Result<Instruction, DecodeError> {
        let bits = word.to_bits();
        // The loader rejects overlapping encodings, so at most one matches.
        let spec = self
            .instructions
            .iter()
            .find(|spec| bits & spec.mask == spec.pattern)
            .ok_or(DecodeError::InvalidEncoding)?;
        let mut fields: Vec<(&str, Operand)> =
            Vec::with_capacity(spec.fields.len());

        for field in &spec.fields {
            if let Some(operand) = self.decode_field(field, bits)? {
                fields.push((field.name.as_str(), operand));
            }
        }

        build(&spec.name, &fields)
    }

    /// Decodes little-endian `bytes` into an instruction of the table.
    ///
    /// # Errors
    ///
    /// Returns the error [`Isa::decode`] reports.
    pub fn decode_bytes(
        &self,
        bytes: [u8; InstructionWord::BYTES],
    ) -> Result<Instruction, DecodeError> {
        self.decode(InstructionWord::from_le_bytes(bytes))
    }

    fn encode_fields(
        &self,
        spec: &InstructionSpec,
        operands: &[(&str, Operand)],
    ) -> Option<u64> {
        let mut bits = spec.pattern;
        let mut used = vec![false; operands.len()];

        for field in &spec.fields {
            if matches!(field.kind, FieldKind::Fixed(_)) {
                continue;
            }

            let index =
                operands.iter().position(|(name, _)| *name == field.name)?;

            used[index] = true;
            bits |= self.operand_bits(field, operands[index].1)? << field.lsb;
        }

        used.iter().all(|flag| *flag).then_some(bits)
    }

    fn operand_bits(&self, field: &Field, operand: Operand) -> Option<u64> {
        let value = match (field.kind, operand) {
            (FieldKind::Type, Operand::Type(kind)) => {
                *self.types.get(kind.name())?
            }
            (FieldKind::Rounding, Operand::Rounding(mode)) => {
                *self.roundings.get(mode.name())?
            }
            (FieldKind::Register, Operand::Register(index)) => index as u64,
            (FieldKind::Bool, Operand::Bool(flag)) => u64::from(flag),
            _ => return None,
        };

        fits(value, field.width).then_some(value)
    }

    fn decode_field(
        &self,
        field: &Field,
        bits: u64,
    ) -> Result<Option<Operand>, DecodeError> {
        let raw = (bits >> field.lsb) & width_mask(field.width);
        let operand = match field.kind {
            FieldKind::Fixed(_) => return Ok(None),
            FieldKind::Type => Operand::Type(self.type_of(field, raw)?),
            FieldKind::Rounding => {
                Operand::Rounding(self.rounding_of(field, raw)?)
            }
            FieldKind::Register => Operand::Register(
                usize::try_from(raw)
                    .map_err(|_| DecodeError::InvalidEncoding)?,
            ),
            FieldKind::Bool => Operand::Bool(raw != 0),
        };

        Ok(Some(operand))
    }

    fn type_of(&self, field: &Field, code: u64) -> Result<Type, DecodeError> {
        let name =
            name_of(&self.types, code).ok_or(DecodeError::InvalidEncoding)?;

        Type::from_name(name).ok_or_else(|| missing_operand(field))
    }

    fn rounding_of(
        &self,
        field: &Field,
        code: u64,
    ) -> Result<RoundingMode, DecodeError> {
        let name = name_of(&self.roundings, code)
            .ok_or(DecodeError::InvalidEncoding)?;

        RoundingMode::from_name(name).ok_or_else(|| missing_operand(field))
    }
}

fn name_of(codes: &BTreeMap<String, u64>, code: u64) -> Option<&str> {
    codes
        .iter()
        .find_map(|(name, value)| (*value == code).then_some(name.as_str()))
}

fn missing_operand(field: &Field) -> DecodeError {
    DecodeError::UnsupportedOperand(field.name.clone())
}

fn visit(
    instruction: &Instruction,
    mut operand: impl FnMut(&'static str, Operand),
) {
    match *instruction {
        Instruction::Add {
            kind,
            rounding,
            sat,
            rd,
            rs1,
            rs2,
        } => {
            operand("type", Operand::Type(kind));
            operand("rounding", Operand::Rounding(rounding));
            operand("sat", Operand::Bool(sat));
            operand("rd", Operand::Register(rd));
            operand("rs1", Operand::Register(rs1));
            operand("rs2", Operand::Register(rs2));
        }
    }
}

fn build(
    name: &str,
    fields: &[(&str, Operand)],
) -> Result<Instruction, DecodeError> {
    match name {
        "add" => build_add(fields),
        _ => Err(DecodeError::UnsupportedInstruction(name.to_owned())),
    }
}

fn build_add(fields: &[(&str, Operand)]) -> Result<Instruction, DecodeError> {
    let mut rest = fields.to_vec();
    let kind = take(&mut rest, "type", Operand::type_of)?;
    let rounding = take(&mut rest, "rounding", Operand::rounding_of)?;
    let sat = take(&mut rest, "sat", Operand::bool_of)?;
    let rd = take(&mut rest, "rd", Operand::register_of)?;
    let rs1 = take(&mut rest, "rs1", Operand::register_of)?;
    let rs2 = take(&mut rest, "rs2", Operand::register_of)?;

    if let Some((name, _)) = rest.first() {
        return Err(DecodeError::UnsupportedOperand((*name).to_owned()));
    }

    Ok(Instruction::Add {
        kind,
        rounding,
        sat,
        rd,
        rs1,
        rs2,
    })
}

fn take<T>(
    fields: &mut Vec<(&str, Operand)>,
    name: &str,
    pick: impl Fn(Operand) -> Option<T>,
) -> Result<T, DecodeError> {
    let index = fields
        .iter()
        .position(|(key, _)| *key == name)
        .ok_or_else(|| DecodeError::UnsupportedOperand(name.to_owned()))?;
    let (_, operand) = fields.remove(index);

    pick(operand)
        .ok_or_else(|| DecodeError::UnsupportedOperand(name.to_owned()))
}

#[cfg(test)]
mod tests {
    use crate::isa::{
        DecodeError, Instruction, InstructionWord, Isa, RoundingMode, Type,
    };

    const TYPES: [Type; 14] = [
        Type::U64,
        Type::I64,
        Type::U32,
        Type::I32,
        Type::U16,
        Type::I16,
        Type::U8,
        Type::I8,
        Type::F32,
        Type::Tf32,
        Type::F16,
        Type::Bf16,
        Type::F8e5m2,
        Type::F8e4m3,
    ];

    const ROUNDINGS: [RoundingMode; 6] = [
        RoundingMode::None,
        RoundingMode::Rne,
        RoundingMode::Rtz,
        RoundingMode::Rdn,
        RoundingMode::Rup,
        RoundingMode::Rmm,
    ];

    const KNOWN_WORD: u64 = 0x0080_0108_3000_0000;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn add(
        kind: Type,
        rounding: RoundingMode,
        sat: bool,
        rd: usize,
        rs1: usize,
        rs2: usize,
    ) -> Instruction {
        Instruction::Add {
            kind,
            rounding,
            sat,
            rd,
            rs1,
            rs2,
        }
    }

    #[test]
    fn known_add_word_round_trips() -> TestResult {
        let isa = Isa::load()?;
        let word = InstructionWord::from_bits(KNOWN_WORD);
        let bytes = [0x00, 0x00, 0x00, 0x30, 0x08, 0x01, 0x80, 0x00];
        let instruction = add(Type::U64, RoundingMode::None, false, 1, 2, 3);

        assert_eq!(isa.encode(&instruction), Some(word));
        assert_eq!(isa.encode_bytes(&instruction), Some(bytes));
        assert_eq!(isa.decode(word), Ok(instruction));
        assert_eq!(isa.decode_bytes(bytes), Ok(instruction));

        Ok(())
    }

    #[test]
    fn every_type_and_rounding_round_trips() -> TestResult {
        let isa = Isa::load()?;

        for kind in TYPES {
            for rounding in ROUNDINGS {
                for sat in [false, true] {
                    let instruction = add(kind, rounding, sat, 61, 17, 42);
                    let label = format!("{kind:?} {rounding:?} sat {sat}");
                    let round_trip = isa
                        .encode(&instruction)
                        .and_then(|word| isa.decode(word).ok());

                    assert_eq!(
                        round_trip,
                        Some(instruction),
                        "{label} did not round trip"
                    );
                }
            }
        }

        Ok(())
    }

    #[test]
    fn invalid_encodings_do_not_decode() -> TestResult {
        let isa = Isa::load()?;
        let base = KNOWN_WORD;
        let mut words = vec![
            base | (14 << 50),
            base | (31 << 50),
            base | (6 << 47),
            base | (7 << 47),
            base | (2 << 55),
            base & !(1 << 55),
        ];

        words.extend((0..28).map(|bit| base | (1 << bit)));

        for word in words {
            assert_eq!(
                isa.decode(InstructionWord::from_bits(word)),
                Err(DecodeError::InvalidEncoding),
                "{word:#018x} decoded"
            );
        }

        Ok(())
    }

    #[test]
    fn registers_outside_the_field_do_not_encode() -> TestResult {
        let isa = Isa::load()?;
        let cases = [(64, 2, 3), (1, 64, 3), (1, 2, 64)];

        for (rd, rs1, rs2) in cases {
            let instruction =
                add(Type::U64, RoundingMode::None, false, rd, rs1, rs2);

            assert_eq!(isa.encode(&instruction), None, "r{rd} r{rs1} r{rs2}");
        }

        let highest = add(Type::U64, RoundingMode::None, false, 63, 63, 63);
        let round_trip =
            isa.encode(&highest).and_then(|word| isa.decode(word).ok());

        assert_eq!(round_trip, Some(highest));

        Ok(())
    }
}
