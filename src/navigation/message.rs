#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::ParsingError;

/// Support Navigation Messages.
/// Refer to [Bibliography::RINEX4] definitions.
#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NavMessageType {
    /// Legacy NAV message
    #[default]
    LNAV,
    /// Glonass FDMA message
    FDMA,
    /// Galileo FNAV message
    FNAV,
    /// Galileo INAV message
    INAV,
    /// IFNV,
    IFNV,
    /// BeiDou D1 NAV message
    D1,
    /// BeiDou D2 NAV message
    D2,
    /// D1D2
    D1D2,
    /// SBAS NAV message
    SBAS,
    /// GPS / QZSS Civilian NAV message
    CNAV,
    /// BeiDou CNV1 message
    CNV1,
    /// GPS / QZSS / BeiDou CNV2 message
    CNV2,
    /// BeiDou CNV3 message
    CNV3,
    /// CNVX special marker
    CNVX,
}

impl std::str::FromStr for NavMessageType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.to_uppercase();
        let c = c.trim();
        match c {
            "LNAV" => Ok(Self::LNAV),
            "FDMA" => Ok(Self::FDMA),
            "FNAV" => Ok(Self::FNAV),
            "INAV" => Ok(Self::INAV),
            "IFNV" => Ok(Self::IFNV),
            "D1" => Ok(Self::D1),
            "D2" => Ok(Self::D2),
            "D1D2" => Ok(Self::D1D2),
            "SBAS" => Ok(Self::SBAS),
            "CNAV" => Ok(Self::CNAV),
            "CNV1" => Ok(Self::CNV1),
            "CNV2" => Ok(Self::CNV2),
            "CNV3" => Ok(Self::CNV3),
            "CNVX" => Ok(Self::CNVX),
            _ => Err(ParsingError::NavMsgType),
        }
    }
}

impl std::fmt::Display for NavMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::LNAV => write!(f, "LNAV"),
            Self::FNAV => write!(f, "FNAV"),
            Self::INAV => write!(f, "INAV"),
            Self::FDMA => write!(f, "FDMA"),
            Self::IFNV => write!(f, "IFNV"),
            Self::D1 => write!(f, "D1"),
            Self::D2 => write!(f, "D2"),
            Self::D1D2 => write!(f, "D1D2"),
            Self::SBAS => write!(f, "SBAS"),
            Self::CNAV => write!(f, "CNAV"),
            Self::CNV1 => write!(f, "CNV1"),
            Self::CNV2 => write!(f, "CNV2"),
            Self::CNV3 => write!(f, "CNV3"),
            Self::CNVX => write!(f, "CNVX"),
        }
    }
}
