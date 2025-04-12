//! Navigation module
mod earth_orientation;
mod ephemeris;
mod frame;
mod header;
mod ionosphere;
mod message;
mod parsing;
mod rinex;
mod time;

pub(crate) mod formatting;

pub(crate) use formatting::format;
pub(crate) use parsing::{is_new_epoch, parse_epoch};

pub use crate::navigation::{
    earth_orientation::EarthOrientation,
    ephemeris::Ephemeris,
    frame::{NavFrame, NavFrameType},
    header::HeaderFields,
    ionosphere::{BdModel, IonosphereModel, KbModel, KbRegionCode, NgModel, NgRegionFlags},
    message::NavMessageType,
    time::TimeOffset,
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

use crate::prelude::{Epoch, SV};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavKey {
    /// [Epoch] of publication
    pub epoch: Epoch,
    /// [SV] source
    pub sv: SV,
    /// [NavMessageType] associated to following [NavFrame]
    pub msgtype: NavMessageType,
    /// [NavFrame] type following
    pub frmtype: NavFrameType,
}

/// Navigation data are [NavFrame]s indexed by [NavKey]
pub type Record = BTreeMap<NavKey, NavFrame>;
