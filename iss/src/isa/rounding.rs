//! Rounding modes of the simulated core.

/// Rounding mode applied by an instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundingMode {
    /// The instruction does not round.
    None,
    /// Round to nearest, ties to even.
    Rne,
    /// Round towards zero.
    Rtz,
    /// Round down, towards negative infinity.
    Rdn,
    /// Round up, towards positive infinity.
    Rup,
    /// Round to nearest, ties away from zero.
    Rmm,
}

impl RoundingMode {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Rne => "rne",
            Self::Rtz => "rtz",
            Self::Rdn => "rdn",
            Self::Rup => "rup",
            Self::Rmm => "rmm",
        }
    }

    pub(super) fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "none" => Self::None,
            "rne" => Self::Rne,
            "rtz" => Self::Rtz,
            "rdn" => Self::Rdn,
            "rup" => Self::Rup,
            "rmm" => Self::Rmm,
            _ => return None,
        })
    }
}
