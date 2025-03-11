//! IONEX module
use std::collections::BTreeMap;

mod coordinates;
mod formatting;
mod grid;
mod header;
mod ipp;
mod mapf;
mod parsing;
mod quantized;
mod rinex;
mod system;
mod tec;

#[cfg(feature = "processing")]
mod decim;

#[cfg(feature = "processing")]
mod mask;

#[cfg(feature = "processing")]
mod repair;

#[cfg(feature = "processing")]
pub(crate) use decim::decim_mut;

#[cfg(feature = "processing")]
pub(crate) use mask::mask_mut;

#[cfg(feature = "processing")]
pub(crate) use repair::repair_mut;

pub use coordinates::QuantizedCoordinates;
pub use grid::Grid;
pub use header::HeaderFields;
pub use ipp::IPPCoordinates;
pub use mapf::MappingFunction;
pub use system::RefSystem;
pub use tec::TEC;

pub(crate) use parsing::{
    is_new_height_map,
    is_new_rms_map,
    is_new_tec_map,
    //parse_height_map,
    parse_rms_map,
    parse_tec_map,
};

pub(crate) use quantized::Quantized;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Epoch, SV};

/// Modeled Ionosphere characteristics
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IonosphereParameters {
    /// Amplitude of the ionospheric delay (s)
    pub amplitude_s: f64,
    /// Period of the ionospheric delay (s)
    pub period_s: f64,
    /// Phase of the ionospheric delay (rad)
    pub phase_rad: f64,
    /// Slant factor is the projection factor
    /// from a vertical signal propagation,
    /// to actual angled shifted propagation (no unit)
    pub slant_factor: f64,
}

/// Possible source of DCBs
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BiasSource {
    /// Referenced against a given vehicle
    SpaceVehicle(SV),
    /// Referenced for an observation station on Earth
    Station(String),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IonexKey {
    /// [Epoch] of this [TEC] estimate.
    pub epoch: Epoch,
    /// [IonexMapCoordinates] to which the [TEC] estimate applies.
    pub coordinates: QuantizedCoordinates,
}

/// IONEX Record is [TEC] maps sorted by [IonexKey]
pub type Record = BTreeMap<IonexKey, TEC>;

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_mapping_func() {
        let content = "COSZ";
        let func = MappingFunction::from_str(content);
        assert!(func.is_ok());
        assert_eq!(func.unwrap(), MappingFunction::CosZ);
        let content = "QFAC";
        let func = MappingFunction::from_str(content);
        assert!(func.is_ok());
        assert_eq!(func.unwrap(), MappingFunction::QFac);
        let content = "DONT";
        let func = MappingFunction::from_str(content);
        assert!(func.is_err());
    }
}
