//! Antex - special RINEX type specific structures
use crate::carrier::Carrier;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Pattern {
    /// Non azimuth dependent pattern
    NonAzimuthDependent(Vec<f64>),
    /// Azimuth dependent pattern
    AzimuthDependent((f64, Vec<f64>)),
}

impl Default for Pattern {
    fn default() -> Self {
        Self::NonAzimuthDependent(Vec::new())
    }
}

impl Pattern {
    /// Returns true if this phase pattern is azimuth dependent
    pub fn is_azimuth_dependent(&self) -> bool {
        matches!(self, Self::AzimuthDependent(_))
    }
    /// Unwraps pattern values, whether it is
    /// Azimuth dependent or not
    pub fn pattern(&self) -> Vec<f64> {
        match self {
            Self::AzimuthDependent((_, values)) => values.clone(),
            Self::NonAzimuthDependent(values) => values.clone(),
        }
    }
    /// Unwraps pattern and associated azimuth angle,
    /// in case of azimuth dependent phase pattern
    pub fn azimuth_pattern(&self) -> Option<(f64, Vec<f64>)> {
        match self {
            Self::AzimuthDependent((angle, values)) => Some((*angle, values.clone())),
            _ => None,
        }
    }
}

/// Describes "frequency" data attached to a specific Antenna
/// in the ATX record
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Frequency {
    /// Carrier, example: "L1", "L2" for GPS, "E1", "E5" for GAL...
    pub carrier: Carrier,
    /// Northern component of the mean antenna phase center
    /// relative to the antenna reference point (ARP), in `mm`.
    pub north: f64,
    /// Eastern component of the mean antenna phase center
    /// relative to the antenna reference point (ARP), in `mm`.
    pub east: f64,
    /// Z component of the mean antenna phase center relative
    /// to the antenna reference point (ARP), in `mm`.
    pub up: f64,
    /// Phase pattern, values in `mm` from antenna.zen1 to antenna.zen2
    /// with increment antenna.dzen, can either be azimuth or non azimmuth dependent
    pub patterns: Vec<Pattern>,
}

impl Default for Frequency {
    fn default() -> Self {
        Self {
            carrier: Carrier::default(),
            north: 0.0_f64,
            east: 0.0_f64,
            up: 0.0_f64,
            patterns: Vec::new(),
        }
    }
}

impl Frequency {
    // /// Returns ARP in geodetic coordinates expressed in decimal degrees.
    // /// Reference point must be in the same coordinates system.
    // pub fn arp_geodetic(&self, ref_pos: (f64, f64, f64)) -> (f64, f64, f64) {
    //     map_3d::enu2geodetic(
    //         self.east * 10.0E-3,
    //         self.north * 10.0E-3,
    //         self.up * 10.0E-3,
    //         ref_pos.0,
    //         ref_pos.1,
    //         ref_pos.2,
    //         map_3d::Ellipsoid::WGS84,
    //     )
    // }

    // /// Returns ARP coordinates in ECEF system.
    // pub fn arp_ecef(&self, ref_pos: (f64, f64, f64)) -> (f64, f64, f64) {
    //     map_3d::enu2ecef(
    //         self.east * 10.0E-3,
    //         self.north * 10.0E-3,
    //         self.up * 10.0E-3,
    //         ref_pos.0,
    //         ref_pos.1,
    //         ref_pos.2,
    //         map_3d::Ellipsoid::WGS84,
    //     )
    // }

    // /// Returns ARP coordinates as North Earth Down coordinates
    // pub fn arp_ned(&self, ref_pos: (f64, f64, f64)) -> (f64, f64, f64) {
    //     let ecef = map_3d::enu2ecef(
    //         self.east * 10.0E-3,
    //         self.north * 10.0E-3,
    //         self.up * 10.0E-3,
    //         ref_pos.0,
    //         ref_pos.1,
    //         ref_pos.2,
    //         map_3d::Ellipsoid::WGS84,
    //     );
    //     map_3d::ecef2ned(
    //         ecef.0,
    //         ecef.1,
    //         ecef.2,
    //         ref_pos.0,
    //         ref_pos.1,
    //         ref_pos.2,
    //         map_3d::Ellipsoid::WGS84,
    //     )
    // }

    pub fn with_carrier(&self, carrier: Carrier) -> Self {
        let mut f = self.clone();
        f.carrier = carrier;
        f
    }
    pub fn with_northern_eccentricity(&self, north: f64) -> Self {
        let mut f = self.clone();
        f.north = north;
        f
    }
    pub fn with_eastern_eccentricity(&self, east: f64) -> Self {
        let mut f = self.clone();
        f.east = east;
        f
    }
    pub fn with_upper_eccentricity(&self, up: f64) -> Self {
        let mut f = self.clone();
        f.up = up;
        f
    }
    pub fn add_pattern(&self, p: Pattern) -> Self {
        let mut f = self.clone();
        f.patterns.push(p.clone());
        f
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_pattern() {
        let default = Pattern::default();
        assert_eq!(default, Pattern::NonAzimuthDependent(Vec::new()));
        assert!(!default.is_azimuth_dependent());
    }
    #[test]
    fn test_frequency() {
        let default = Frequency::default();
        assert_eq!(default.carrier, Carrier::default());
        assert_eq!(default.north, 0.0_f64);
        assert_eq!(default.east, 0.0_f64);
        assert_eq!(default.up, 0.0_f64);
    }
}
