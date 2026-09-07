//! Navigation module
pub mod orbits;

mod earth_orientation;
mod ephemeris;
mod formatting;
mod frame;
mod health;
mod ionosphere;
mod message;
mod parsing;
mod rinex;
mod system_time;

pub(crate) use formatting::format;
pub(crate) use parsing::{is_new_epoch, parse_epoch};

pub use crate::navigation::{
    earth_orientation::EarthOrientation,
    ephemeris::Ephemeris,
    frame::{NavFrame, NavFrameType},
    health::{GeoHealth, GloHealth, Health, IrnssHealth},
    ionosphere::{
        BdModel, GloCdmaModel, IonosphereModel, KbModel, KbRegionCode, NavicKbModel,
        NavicNeqnModel, NavicNeqnRegion, NgModel, NgRegionFlags,
    },
    message::{NavMessageSubtype, NavMessageType},
    orbits::OrbitItem,
    system_time::SystemTime,
};

#[cfg(feature = "processing")]
pub(crate) mod mask; // mask Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod decim; // decim Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod repair; // repair Trait implementation

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use crate::prelude::{Constellation, Epoch, ParsingError, TimeScale, SV};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavKey {
    /// [Epoch] of publication
    pub epoch: Epoch,
    /// [SV] source
    pub sv: SV,
    /// [NavMessageType] associated to following [NavFrame]
    pub msgtype: NavMessageType,
    /// [NavMessageSubtype] (RINEX 4.02), only defined for a few ION messages
    pub subtype: Option<NavMessageSubtype>,
    /// [NavFrame] type following
    pub frmtype: NavFrameType,
}

/// Navigation data are [NavFrame]s indexed by [NavKey]
pub type Record = BTreeMap<NavKey, NavFrame>;

/// [TimeScale] in which the navigation epochs of this [Constellation]
/// are expressed.
pub(crate) fn timescale(constellation: Constellation) -> Result<TimeScale, ParsingError> {
    match constellation {
        // NavIC time counts weeks from the GPS epoch and does not
        // apply leap seconds: equivalent to GPST.
        Constellation::IRNSS => Ok(TimeScale::GPST),
        c => c.timescale().ok_or(ParsingError::NoTimescaleDefinition),
    }
}
