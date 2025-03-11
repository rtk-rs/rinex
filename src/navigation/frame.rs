use crate::{
    navigation::{EarthOrientation, Ephemeris, IonosphereModel, SystemTime},
    prelude::ParsingError,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Navigation Frame classes
#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NavFrameType {
    /// Ephemeris frame
    #[default]
    Ephemeris,
    /// System Time Offset frame
    SystemTimeOffset,
    /// Earth Orientation frame
    EarthOrientation,
    /// Ionosphere model
    IonosphereModel,
}

impl std::fmt::Display for NavFrameType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ephemeris => f.write_str("EPH"),
            Self::SystemTimeOffset => f.write_str("STO"),
            Self::EarthOrientation => f.write_str("EOP"),
            Self::IonosphereModel => f.write_str("ION"),
        }
    }
}

impl std::str::FromStr for NavFrameType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.to_uppercase();
        let c = c.trim();
        match c {
            "EPH" => Ok(Self::Ephemeris),
            "STO" => Ok(Self::SystemTimeOffset),
            "EOP" => Ok(Self::EarthOrientation),
            "ION" => Ok(Self::IonosphereModel),
            _ => Err(ParsingError::NavFrameClass),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum NavFrame {
    EPH(Ephemeris),
    EOP(EarthOrientation),
    ION(IonosphereModel),
    STO(SystemTime),
}

impl NavFrame {
    /// [Ephemeris] unwrapping attempt.
    pub fn as_ephemeris(&self) -> Option<&Ephemeris> {
        match self {
            Self::EPH(eph) => Some(eph),
            _ => None,
        }
    }

    /// Mutable [Ephemeris] unwrapping attempt.
    pub fn as_mut_ephemeris(&mut self) -> Option<&mut Ephemeris> {
        match self {
            Self::EPH(eph) => Some(eph),
            _ => None,
        }
    }

    /// [SystemTime] unwrapping attempt.
    pub fn as_system_time(&self) -> Option<&SystemTime> {
        match self {
            Self::STO(fr) => Some(fr),
            _ => None,
        }
    }

    /// Mutable [SystemTime] unwrapping attempt.
    pub fn as_mut_system_time(&mut self) -> Option<&mut SystemTime> {
        match self {
            Self::STO(fr) => Some(fr),
            _ => None,
        }
    }

    pub fn as_ionosphere_model(&self) -> Option<&IonosphereModel> {
        match self {
            Self::ION(fr) => Some(fr),
            _ => None,
        }
    }

    pub fn as_mut_ionosphere_model(&mut self) -> Option<&mut IonosphereModel> {
        match self {
            Self::ION(fr) => Some(fr),
            _ => None,
        }
    }

    /// [EarthOrientation] unwrapping attempt
    pub fn as_earth_orientation(&self) -> Option<&EarthOrientation> {
        match self {
            Self::EOP(fr) => Some(fr),
            _ => None,
        }
    }

    /// Mutable [EarthOrientation] unwrapping attempt
    pub fn as_mut_earth_orientation(&mut self) -> Option<&mut EarthOrientation> {
        match self {
            Self::EOP(fr) => Some(fr),
            _ => None,
        }
    }
}
