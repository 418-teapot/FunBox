//! Operand types of the simulated core.

/// Operand type of an instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    /// 64-bit signed integer.
    I64,
    /// 64-bit unsigned integer.
    U64,
    /// 32-bit signed integer.
    I32,
    /// 32-bit unsigned integer.
    U32,
    /// 16-bit signed integer.
    I16,
    /// 16-bit unsigned integer.
    U16,
    /// 8-bit signed integer.
    I8,
    /// 8-bit unsigned integer.
    U8,
    /// IEEE 754 binary32 floating point.
    F32,
    /// TensorFloat-32: binary32 with a 10-bit significand.
    Tf32,
    /// IEEE 754 binary16 floating point.
    F16,
    /// Brain floating point: binary16 with 8 significand bits.
    Bf16,
    /// OCP FP8 E5M2 floating point.
    F8e5m2,
    /// OCP FP8 E4M3 floating point.
    F8e4m3,
}

impl Type {
    /// Returns whether the type is a floating-point type.
    #[must_use]
    pub const fn is_float(self) -> bool {
        matches!(
            self,
            Self::F32
                | Self::Tf32
                | Self::F16
                | Self::Bf16
                | Self::F8e5m2
                | Self::F8e4m3
        )
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::I16 => "i16",
            Self::U16 => "u16",
            Self::I8 => "i8",
            Self::U8 => "u8",
            Self::F32 => "f32",
            Self::Tf32 => "tf32",
            Self::F16 => "f16",
            Self::Bf16 => "bf16",
            Self::F8e5m2 => "f8e5m2",
            Self::F8e4m3 => "f8e4m3",
        }
    }

    pub(super) fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "i64" => Self::I64,
            "u64" => Self::U64,
            "i32" => Self::I32,
            "u32" => Self::U32,
            "i16" => Self::I16,
            "u16" => Self::U16,
            "i8" => Self::I8,
            "u8" => Self::U8,
            "f32" => Self::F32,
            "tf32" => Self::Tf32,
            "f16" => Self::F16,
            "bf16" => Self::Bf16,
            "f8e5m2" => Self::F8e5m2,
            "f8e4m3" => Self::F8e4m3,
            _ => return None,
        })
    }
}
