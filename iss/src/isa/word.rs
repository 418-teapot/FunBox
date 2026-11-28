//! Encoded instruction words.

/// Encoded form of one instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstructionWord(u64);

impl InstructionWord {
    /// Width of a word in bits.
    pub const BITS: u32 = u64::BITS;

    /// Width of a word in bytes.
    pub const BYTES: usize = std::mem::size_of::<u64>();

    /// Wraps `bits` into a word.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Returns the bits of the word.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    /// Reads a word from its little-endian bytes.
    #[must_use]
    pub const fn from_le_bytes(bytes: [u8; Self::BYTES]) -> Self {
        Self(u64::from_le_bytes(bytes))
    }

    /// Returns the little-endian bytes of the word.
    #[must_use]
    pub const fn to_le_bytes(self) -> [u8; Self::BYTES] {
        self.0.to_le_bytes()
    }
}
