//! Instruction set architecture of the simulated core.

mod codec;
mod instruction;
mod parser;
mod rounding;
mod spec;
mod r#type;
mod word;

pub use self::codec::DecodeError;
pub use self::instruction::Instruction;
pub use self::parser::IsaError;
pub use self::rounding::RoundingMode;
pub use self::spec::Isa;
pub use self::r#type::Type;
pub use self::word::InstructionWord;
