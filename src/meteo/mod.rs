//! Meteo RINEX module
mod formatting;
mod header;
mod parsing;
mod rinex;
mod sensor; // high level methods

pub use header::HeaderFields;
pub use sensor::Sensor;

use crate::prelude::{Epoch, Observable};
use std::collections::BTreeMap;

pub(crate) use formatting::format;
pub(crate) use parsing::{is_new_epoch, parse_epoch};

#[cfg(feature = "processing")]
pub(crate) mod mask; // mask Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod decim; // decim Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod repair; // repair Trait implementation

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MeteoKey {
    /// [Epoch] of observation
    pub epoch: Epoch,
    /// [Observable] determines the physics
    pub observable: Observable,
}

/// [MeteoObservation]s sorted by [Epoch]
pub type Record = BTreeMap<MeteoKey, f64>;
