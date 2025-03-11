//! Geodetic marker

use crate::{
    fmt_rinex,
    prelude::{FormattingError, ParsingError},
};

use std::io::{BufWriter, Write};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeodeticMarker {
    /// Marker name
    pub name: String,
    /// Marker type
    pub marker_type: Option<MarkerType>,
    /// Marker/monument ID
    identification: Option<u32>,
    /// Marker/monument number.
    /// Probably if agency has more than one of them.
    monument: Option<u16>,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MarkerType {
    /// Earth fixed & high precision
    #[default]
    Geodetic,
    /// Earth fixed & low precision
    NonGeodetic,
    /// Generated from network
    NonPhysical,
    /// Orbiting space vehicle
    Spaceborne,
    /// Aircraft, balloon..
    Airborne,
    /// Mobile water craft
    Watercraft,
    /// Mobile terrestrial vehicle
    Groundcraft,
    /// Fixed on water surface
    FixedBuoy,
    /// Floating on water surface
    FloatingBuoy,
    /// Floating on ice
    FloatingIce,
    /// Fixed on glacier
    Glacier,
    /// Rockets, shells, etc..
    Ballistic,
    /// Animal carrying a receiver
    Animal,
    /// Human being carrying a receiver
    Human,
}

impl std::str::FromStr for MarkerType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "geodetic" => Ok(Self::Geodetic),
            "non geodetic" => Ok(Self::NonGeodetic),
            "ground craft" => Ok(Self::Groundcraft),
            "water craft" => Ok(Self::Watercraft),
            "airborne" => Ok(Self::Airborne),
            "non physical" => Ok(Self::NonPhysical),
            "spaceborne" => Ok(Self::Spaceborne),
            "floating ice" => Ok(Self::FloatingIce),
            "floating buoy" => Ok(Self::FloatingBuoy),
            "glacier" => Ok(Self::Glacier),
            "ballistic" => Ok(Self::Ballistic),
            "animal" => Ok(Self::Animal),
            "human" => Ok(Self::Human),
            _ => Err(ParsingError::MarkerType),
        }
    }
}

impl std::fmt::Display for MarkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Geodetic => write!(f, "GEODETIC"),
            _ => write!(f, "HUMAN"),
        }
    }
}

impl GeodeticMarker {
    /// Formats [GeodeticMarker] into [BufWriter]
    pub(crate) fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        writeln!(w, "{}", fmt_rinex(&self.name, "MARKER NAME"))?;
        if let Some(number) = self.number() {
            writeln!(w, "{}", fmt_rinex(&number, "MARKER NUMBER"))?;
        }
        if let Some(marker_type) = &self.marker_type {
            writeln!(w, "{}", fmt_rinex(&marker_type.to_string(), "MARKER TYPE"))?;
        }
        Ok(())
    }

    /// Returns a GeodeticMarker with given "name".
    pub fn with_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.name = name.to_string();
        s
    }
    /// Returns marker "number" in standardized format
    pub fn number(&self) -> Option<String> {
        let id = self.identification?;
        let monument = self.monument?;
        Some(format!("{:05}M{:03}", id, monument))
    }
    /// Returns a GeodeticMarker with "number" only if it matches standardized format.
    pub fn with_number(&self, content: &str) -> Self {
        let mut s = self.clone();
        if content.len() == 9 && content.chars().nth(5) == Some('M') {
            if let Ok(id) = u32::from_str_radix(&content[..5], 10) {
                if let Ok(monument) = u16::from_str_radix(&content[7..], 10) {
                    s.identification = Some(id);
                    s.monument = Some(monument);
                }
            }
        }
        s
    }
}

#[cfg(test)]
mod test {
    use super::GeodeticMarker;
    #[test]
    fn marker_number() {
        let marker = GeodeticMarker::default();
        let marker = marker.with_number("10118M001");
        assert_eq!(marker.number(), Some("10118M001".to_string()));
    }
}
