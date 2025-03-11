//! DORIS module
use thiserror::Error;

use crate::{
    observable::Observable, observation::ClockObservation, observation::EpochFlag, prelude::Epoch,
};

use std::collections::{BTreeMap, HashMap};

use gnss_rs::domes::Error as DomesParsingError;

mod formatting;
mod header;
mod parsing;
mod rinex;
mod station;

pub(crate) use formatting::format;
pub(crate) use parsing::{is_new_epoch, parse_epoch};

#[cfg(feature = "processing")]
pub(crate) mod decim;

#[cfg(feature = "processing")]
pub(crate) mod mask;

#[cfg(feature = "processing")]
pub(crate) mod repair;

pub use header::HeaderFields;
pub use station::Station;

/// DORIS Station & record parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid station")]
    InvalidStation,
    #[error("failed to parse station id")]
    IdParsing,
    #[error("invalid station DOMES code")]
    DomesError(#[from] DomesParsingError),
    #[error("failed to parse beacon generation")]
    BeaconGenerationParsing,
    #[error("failed to parse `k` factor")]
    KfParsing,
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SignalObservation {
    /// M1 flag
    pub m1: Option<u8>,
    /// M2 flag
    pub m2: Option<u8>,
    /// Actual measurement, unit depends on associated [Observable]
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SignalKey {
    /// [Observable] determines the physics, the signal and signal modulation.
    pub observable: Observable,
    /// [Station] is the signal source
    pub station: Station,
}

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Observations {
    /// DORIS satellite (on board) clock state, at [Epoch] of observation
    pub clock: ClockObservation,
    /// Whether [ClockObservation] was extrapolated or is an actual measurement.
    pub clock_extrapolated: bool,
    /// Observed signals from ground [Station]s, as [SignalObservation]
    pub signals: HashMap<SignalKey, SignalObservation>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DorisKey {
    /// [Epoch] of observation (=sampling)
    pub epoch: Epoch,
    /// [EpochFlag] describing sampling conditions
    pub flag: EpochFlag,
}

/// DORIS Record contains [Observations] sorted by [DorisKey].
pub type Record = BTreeMap<DorisKey, Observations>;
