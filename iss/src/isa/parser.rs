//! Parsing and validation of the ISA table.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

use crate::regs::Regs;

use super::spec::{Field, FieldKind, InstructionSpec, Isa, fits, width_mask};
use super::word::InstructionWord;

const TABLE: &str = "isa/base.toml";

const NAME_KEY: &str = "name";

const NAME_PLACEHOLDER: &str = "{name}";

const TOP_KEYS: [&str; 3] = ["types", "rounding_modes", "instructions"];

const ENTRY_KEYS: [&str; 3] = ["name", "assembly", "fields"];

const FIELD_KEYS: [&str; 5] = ["name", "kind", "msb", "lsb", "value"];

/// Error returned when the ISA table cannot be loaded.
#[derive(Debug)]
pub enum IsaError {
    /// The table cannot be read.
    Io(std::io::Error),
    /// The table is not valid TOML.
    Parse(toml::de::Error),
    /// The table is readable but does not describe a usable ISA.
    Invalid(String),
}

impl fmt::Display for IsaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => {
                write!(formatter, "cannot read the ISA table: {error}")
            }
            Self::Parse(error) => {
                write!(formatter, "cannot parse the ISA table: {error}")
            }
            Self::Invalid(message) => {
                write!(formatter, "invalid ISA table: {message}")
            }
        }
    }
}

impl std::error::Error for IsaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<std::io::Error> for IsaError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for IsaError {
    fn from(error: toml::de::Error) -> Self {
        Self::Parse(error)
    }
}

impl Isa {
    /// Returns the path of the ISA table next to this crate.
    #[must_use]
    pub fn path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(TABLE)
    }

    /// Loads the ISA table next to this crate.
    ///
    /// # Errors
    ///
    /// Returns [`IsaError`] if the table cannot be read, cannot be parsed, or
    /// does not describe a usable ISA.
    pub fn load() -> Result<Self, IsaError> {
        let source = fs::read_to_string(Self::path())?;

        Self::parse(&source)
    }

    fn parse(source: &str) -> Result<Self, IsaError> {
        let document = toml::from_str::<Table>(source)?;

        check_keys(&document, &TOP_KEYS, "isa")?;

        let mut isa = Self {
            types: mapping(&document, "types", "isa")?,
            roundings: mapping(&document, "rounding_modes", "isa")?,
            instructions: Vec::new(),
        };

        isa.load_instructions(&document)?;

        Ok(isa)
    }

    fn load_instructions(&mut self, document: &Table) -> Result<(), IsaError> {
        let instructions = array(document, "instructions", "isa")?;

        if instructions.is_empty() {
            return invalid("isa: instructions must not be empty");
        }

        let mut names: Vec<&str> = Vec::new();

        for (index, value) in instructions.iter().enumerate() {
            let context = format!("instructions[{index}]");
            let Some(entry) = value.as_table() else {
                return invalid(format!("{context}: expected a table"));
            };

            check_keys(entry, &ENTRY_KEYS, &context)?;

            let name = string(entry, "name", &context)?;

            if names.contains(&name) {
                return invalid(format!(
                    "{context}: duplicate instruction {name:?}"
                ));
            }

            names.push(name);

            let spec = self.instruction(name, &context, entry)?;
            let fixed = (spec.mask, spec.pattern);

            if let Some(other) = self
                .instructions
                .iter()
                .find(|other| overlaps((other.mask, other.pattern), fixed))
            {
                return invalid(format!(
                    "{context}: encoding overlaps {:?}",
                    other.name
                ));
            }

            self.instructions.push(spec);
        }

        Ok(())
    }

    fn instruction(
        &self,
        name: &str,
        context: &str,
        entry: &Table,
    ) -> Result<InstructionSpec, IsaError> {
        let assembly = string(entry, "assembly", context)?;

        if !assembly.contains(NAME_PLACEHOLDER) {
            return invalid(format!(
                "{context}: assembly lacks {NAME_PLACEHOLDER:?}"
            ));
        }

        let values = array(entry, "fields", context)?;

        if values.is_empty() {
            return invalid(format!("{context}: fields must not be empty"));
        }

        let fields = self.fields(context, values)?;

        check_placeholders(context, assembly, &fields)?;

        let (mask, pattern) = fixed_pattern(&fields);

        Ok(InstructionSpec {
            name: name.to_owned(),
            fields,
            mask,
            pattern,
        })
    }

    fn fields(
        &self,
        context: &str,
        values: &[Value],
    ) -> Result<Vec<Field>, IsaError> {
        let mut covered = vec![false; InstructionWord::BITS as usize];
        let mut fields: Vec<Field> = Vec::new();

        for (index, value) in values.iter().enumerate() {
            let context = format!("{context}.fields[{index}]");
            let Some(entry) = value.as_table() else {
                return invalid(format!("{context}: expected a table"));
            };

            let field = self.field(&context, entry)?;

            if fields.iter().any(|other| other.name == field.name) {
                return invalid(format!(
                    "{context}: duplicate field {:?}",
                    field.name
                ));
            }

            for bit in field.lsb..field.lsb + field.width {
                if covered[bit as usize] {
                    return invalid(format!("{context}: overlaps bit {bit}"));
                }

                covered[bit as usize] = true;
            }

            fields.push(field);
        }

        if let Some(bit) = covered.iter().position(|covered| !covered) {
            return invalid(format!("{context}: bit {bit} is uncovered"));
        }

        Ok(fields)
    }

    fn field(&self, context: &str, entry: &Table) -> Result<Field, IsaError> {
        check_keys(entry, &FIELD_KEYS, context)?;

        let name = string(entry, "name", context)?;

        if name == NAME_KEY {
            return invalid(format!(
                "{context}: {NAME_KEY:?} is reserved for the instruction"
            ));
        }

        check_identifier(context, NAME_KEY, name)?;

        let msb = position(entry, "msb", context)?;
        let lsb = position(entry, "lsb", context)?;

        if msb >= InstructionWord::BITS || msb < lsb {
            return invalid(format!("{context}: invalid bits {lsb}..={msb}"));
        }

        let width = msb - lsb + 1;
        let kind = self.field_kind(context, entry, width)?;

        Ok(Field {
            name: name.to_owned(),
            lsb,
            width,
            kind,
        })
    }

    fn field_kind(
        &self,
        context: &str,
        entry: &Table,
        width: u32,
    ) -> Result<FieldKind, IsaError> {
        let value = fixed_value(context, entry, width)?;
        let label = optional_string(entry, "kind", context)?;

        match (value, label) {
            (Some(value), None) => Ok(FieldKind::Fixed(value)),
            (Some(_), Some(_)) => invalid(format!(
                "{context}: value must not be combined with kind"
            )),
            (None, None) => {
                invalid(format!("{context}: a field needs kind or value"))
            }
            (None, Some(label)) => self.operand_kind(context, label, width),
        }
    }

    fn operand_kind(
        &self,
        context: &str,
        label: &str,
        width: u32,
    ) -> Result<FieldKind, IsaError> {
        match label {
            "type" => {
                check_width(&self.types, "types", context, width)?;

                Ok(FieldKind::Type)
            }
            "rounding" => {
                let table = "rounding_modes";

                check_width(&self.roundings, table, context, width)?;

                Ok(FieldKind::Rounding)
            }
            "register" if width == Regs::INDEX_BITS => Ok(FieldKind::Register),
            "bool" if width == 1 => Ok(FieldKind::Bool),
            "register" | "bool" => invalid(format!(
                "{context}: {label} needs a fixed width, found {width}"
            )),
            _ => invalid(format!("{context}: unknown kind {label:?}")),
        }
    }
}

fn invalid<T>(message: impl Into<String>) -> Result<T, IsaError> {
    Err(IsaError::Invalid(message.into()))
}

fn check_keys(
    document: &Table,
    allowed: &[&str],
    context: &str,
) -> Result<(), IsaError> {
    for key in document.keys() {
        if !allowed.contains(&key.as_str()) {
            return invalid(format!("{context}: unknown key {key:?}"));
        }
    }

    Ok(())
}

fn string<'a>(
    document: &'a Table,
    key: &str,
    context: &str,
) -> Result<&'a str, IsaError> {
    match document.get(key) {
        Some(Value::String(text)) => Ok(text),
        Some(_) => invalid(format!("{context}: {key} must be a string")),
        None => invalid(format!("{context}: missing {key}")),
    }
}

fn optional_string<'a>(
    document: &'a Table,
    key: &str,
    context: &str,
) -> Result<Option<&'a str>, IsaError> {
    match document.get(key) {
        Some(Value::String(text)) => Ok(Some(text)),
        Some(_) => invalid(format!("{context}: {key} must be a string")),
        None => Ok(None),
    }
}

fn integer(
    document: &Table,
    key: &str,
    context: &str,
) -> Result<u64, IsaError> {
    let Some(value) = document.get(key) else {
        return invalid(format!("{context}: missing {key}"));
    };
    let Some(value) = value.as_integer() else {
        return invalid(format!("{context}: {key} must be an integer"));
    };
    let Ok(value) = u64::try_from(value) else {
        return invalid(format!("{context}: {key} must not be negative"));
    };

    Ok(value)
}

fn position(
    document: &Table,
    key: &str,
    context: &str,
) -> Result<u32, IsaError> {
    let value = integer(document, key, context)?;

    u32::try_from(value).map_err(|_| {
        IsaError::Invalid(format!("{context}: {key} {value} is out of range"))
    })
}

fn array<'a>(
    document: &'a Table,
    key: &str,
    context: &str,
) -> Result<&'a Vec<Value>, IsaError> {
    match document.get(key) {
        Some(Value::Array(items)) => Ok(items),
        Some(_) => invalid(format!("{context}: {key} must be an array")),
        None => invalid(format!("{context}: missing {key}")),
    }
}

fn mapping(
    document: &Table,
    key: &str,
    context: &str,
) -> Result<BTreeMap<String, u64>, IsaError> {
    let Some(value) = document.get(key) else {
        return invalid(format!("{context}: missing {key}"));
    };
    let Some(table) = value.as_table() else {
        return invalid(format!("{context}: {key} must be a table"));
    };
    let mut codes = BTreeMap::new();
    let mut seen: BTreeMap<u64, &str> = BTreeMap::new();

    for (name, value) in table {
        let Some(code) = value.as_integer() else {
            return invalid(format!(
                "{context}: {key}.{name} must be an integer code"
            ));
        };
        let Ok(code) = u64::try_from(code) else {
            return invalid(format!(
                "{context}: {key}.{name} must not be negative"
            ));
        };

        if let Some(other) = seen.insert(code, name) {
            return invalid(format!(
                "{context}: {key}.{name} and {key}.{other} share code {code}"
            ));
        }

        codes.insert(name.clone(), code);
    }

    Ok(codes)
}

fn fixed_value(
    context: &str,
    entry: &Table,
    width: u32,
) -> Result<Option<u64>, IsaError> {
    let Some(value) = entry.get("value") else {
        return Ok(None);
    };
    let Some(value) = value.as_integer() else {
        return invalid(format!("{context}: value must be an integer"));
    };
    let Ok(value) = u64::try_from(value) else {
        return invalid(format!("{context}: value must not be negative"));
    };

    if fits(value, width) {
        Ok(Some(value))
    } else {
        invalid(format!(
            "{context}: value {value} does not fit {width} bits"
        ))
    }
}

fn check_width(
    codes: &BTreeMap<String, u64>,
    table: &str,
    context: &str,
    width: u32,
) -> Result<(), IsaError> {
    if codes.is_empty() {
        return invalid(format!("{context}: {table} codes are missing"));
    }

    for (name, code) in codes {
        if !fits(*code, width) {
            return invalid(format!(
                "{context}: {table} code {name:?} = {code} does not fit \
                 {width} bits"
            ));
        }
    }

    Ok(())
}

fn check_identifier(
    context: &str,
    key: &str,
    text: &str,
) -> Result<(), IsaError> {
    let mut characters = text.chars();
    let head = characters
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_');
    let tail = characters.all(|rest| rest.is_alphanumeric() || rest == '_');

    if head && tail {
        return Ok(());
    }

    invalid(format!("{context}: {key} {text:?} is not an identifier"))
}

fn check_placeholders(
    context: &str,
    assembly: &str,
    fields: &[Field],
) -> Result<(), IsaError> {
    let mut rest = assembly;

    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            return invalid(format!("{context}: unclosed placeholder"));
        };
        let label = &after[..end];
        let known =
            label == NAME_KEY || fields.iter().any(|field| field.name == label);

        if !known {
            return invalid(format!(
                "{context}: assembly placeholder {{{label}}} has no field"
            ));
        }

        rest = &after[end + 1..];
    }

    Ok(())
}

fn fixed_pattern(fields: &[Field]) -> (u64, u64) {
    let mut mask = 0;
    let mut pattern = 0;

    for field in fields {
        if let FieldKind::Fixed(value) = field.kind {
            mask |= width_mask(field.width) << field.lsb;
            pattern |= value << field.lsb;
        }
    }

    (mask, pattern)
}

/// Returns whether one instruction word can match both patterns.
const fn overlaps(first: (u64, u64), second: (u64, u64)) -> bool {
    let (first_mask, first_pattern) = first;
    let (second_mask, second_pattern) = second;

    (first_pattern ^ second_pattern) & first_mask & second_mask == 0
}
