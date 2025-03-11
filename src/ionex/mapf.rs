use crate::prelude::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Mapping function used in when determining this IONEX
pub enum MappingFunction {
    /// 1/cos(z)
    CosZ,
    /// Q-factor
    QFac,
}

impl std::str::FromStr for MappingFunction {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "cosz" => Ok(Self::CosZ),
            "qfac" => Ok(Self::QFac),
            _ => Err(ParsingError::IonexMappingFunction),
        }
    }
}

impl std::fmt::Display for MappingFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CosZ => write!(f, "Cos(z)"),
            Self::QFac => write!(f, "Q-factor"),
        }
    }
}
