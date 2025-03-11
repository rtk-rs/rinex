#[cfg(feature = "serde")]
use serde::Serialize;

mod sv;
pub use sv::SvAntenna;

use crate::{
    linspace::Linspace,
    prelude::{Epoch, ParsingError},
};

/// Known Calibration Methods
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum CalibrationMethod {
    #[default]
    Unknown,
    Chamber,
    Field,
    Robot,
    /// Copied from other antenna
    Copied,
    /// Converted from igs_01.pcv or blank
    Converted,
}

impl std::str::FromStr for CalibrationMethod {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "chamber" => Ok(Self::Chamber),
            "field" => Ok(Self::Field),
            "robot" => Ok(Self::Robot),
            "copied" => Ok(Self::Copied),
            "converted" => Ok(Self::Converted),
            "" => Ok(Self::Unknown),
            _ => Err(ParsingError::AntexCalibrationMethod),
        }
    }
}

/// Calibration information
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Calibration {
    /// Calibration method
    pub method: CalibrationMethod,
    /// Agency who performed this calibration
    pub agency: String,
    /// Date of calibration
    pub date: Epoch,
    /// Number of calibrated antennas
    pub number: u16,
    /// Calibration Validity Period: (Start, End) inclusive
    pub validity_period: Option<(Epoch, Epoch)>,
}

/// Antenna description, as contained in ATX records
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Antenna {
    /// Antenna specific field, either a
    /// spacecraft antenna or a receiver antenna
    pub specific: AntennaSpecific,
    /// Information on the calibration process.
    pub calibration: Calibration,
    /// Zenith grid definition.
    /// The grid is expressed in zenith angles for RxAntenneas,
    /// or in nadir Angle for SvAntennas.
    pub zenith_grid: Linspace,
    /// Azmiuth increment
    pub azi_inc: f64,
    /// SINEX code normalization
    pub sinex_code: String,
}

impl Antenna {
    /// Returns whether this calibration is valid at the current Epoch
    /// or not. Note that specs that did not come with a calibration
    /// certificate validity are always considered valid.
    /// You then need to refer to the date of that calibration (always given)
    /// and should only consider recently calibrated data.
    pub fn is_valid(&self, now: Epoch) -> bool {
        if let Some((from, until)) = self.calibration.validity_period {
            now > from && now < until
        } else {
            true
        }
    }
    // /// Returns the mean phase center position.
    // /// If Self is a Receiver Antenna ([`RxAntenna`]),
    // /// the returned position is expressed as an offset to the
    // /// Antenna Reference Position (ARP).
    // /// If Self is a Spacecraft Antenna ([`SvAntenna`]),
    // /// the returned position is expressed as an offset to the Spacecraft
    // /// Mass Center.
    // fn mean_phase_center(&self, _reference: (f64, f64, f64)) -> (f64, f64, f64) {
    //     (0.0_f64, 0.0_f64, 0.0_f64)
    // }
    /// Builds an Antenna with given Calibration infos
    pub fn with_calibration(&self, calib: Calibration) -> Self {
        let mut a = self.clone();
        a.calibration = calib.clone();
        a
    }
    /// Builds an Antenna with given Zenith Grid
    pub fn with_zenith_grid(&self, grid: Linspace) -> Self {
        let mut a = self.clone();
        a.zenith_grid = grid.clone();
        a
    }
    /// Builds an Antenna with given Validity period
    pub fn with_validity_period(&self, start: Epoch, end: Epoch) -> Self {
        let mut a = self.clone();
        a.calibration.validity_period = Some((start, end));
        a
    }
    /// Builds an Antenna with given Azimuth increment
    pub fn with_dazi(&self, dazi: f64) -> Self {
        let mut a = self.clone();
        a.azi_inc = dazi;
        a
    }
    /// Add custom specificities
    pub fn with_specificities(&self, specs: AntennaSpecific) -> Self {
        let mut a = self.clone();
        a.specific = specs.clone();
        a
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum AntennaSpecific {
    /// Attributes of a receiver antenna
    RxAntenna(RxAntenna),
    /// Attributes of a spacecraft antenna
    SvAntenna(sv::SvAntenna),
}

impl Default for AntennaSpecific {
    fn default() -> Self {
        Self::RxAntenna(RxAntenna::default())
    }
}

/// Antenna Matcher is used to easily locate RX antennas
/// contained in ATX records. AntennaMatcher is case insensitive.
#[derive(Clone, Debug)]
pub enum AntennaMatcher {
    /// Identify an (RX) antenna model by its IGS code
    IGSCode(String),
    /// Identify an (RX) antenna model by its serial number
    SerialNumber(String),
}

impl AntennaMatcher {
    #[cfg(feature = "antex")]
    pub(crate) fn to_lowercase(&self) -> Self {
        match self {
            Self::IGSCode(code) => Self::IGSCode(code.to_lowercase()),
            Self::SerialNumber(sn) => Self::SerialNumber(sn.to_lowercase()),
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RxAntenna {
    /// IGS antenna code
    pub igs_type: String,
    /// Antenna serial number
    pub serial_number: Option<String>,
}
