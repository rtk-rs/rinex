use crate::{
    antex::Record as AntexRecord, clock::Record as ClockRecord, doris::Record as DorisRecord,
    ionex::Record as IonexRecord, meteo::Record as MeteoRecord, navigation::Record as NavRecord,
    observation::Record as ObservationRecord, prelude::Epoch,
};

use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::Serialize;

mod formatting;
mod parsing;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Record {
    /// [AntexRecord] contains antenna calibration profile
    AntexRecord(AntexRecord),
    /// [ClockRecord] contains SV and ground clock states
    ClockRecord(ClockRecord),
    /// IONEX (Ionosphere maps), see [IonexRecord]
    IonexRecord(IonexRecord),
    /// Meteo record, see [MeteoRecord]
    MeteoRecord(MeteoRecord),
    /// Navigation messages stored in [NavRecord]
    NavRecord(NavRecord),
    /// Observation record [ObservationRecord]
    ObsRecord(ObservationRecord),
    /// DORIS RINEX, special DORIS signals observation
    DorisRecord(DorisRecord),
}

/// Record comments are high level informations, sorted by epoch
/// (timestamp) of appearance. We deduce the "associated" timestamp from the
/// previosuly parsed epoch, when parsing the record.
pub type Comments = BTreeMap<Epoch, Vec<String>>;

impl Record {
    /// [AntexRecord] unwrapping attempt.
    pub fn as_antex(&self) -> Option<&AntexRecord> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Mutable [AntexRecord] unwrapping attempt.
    pub fn as_mut_antex(&mut self) -> Option<&mut AntexRecord> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }

    /// [ClockRecord] unwrapping attempt.
    pub fn as_clock(&self) -> Option<&ClockRecord> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Mutable [ClockRecord] unwrapping attempt.
    pub fn as_mut_clock(&mut self) -> Option<&mut ClockRecord> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }

    /// [IonexRecord] unwrapping attempt.
    pub fn as_ionex(&self) -> Option<&IonexRecord> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Mutable [IonexRecord] unwrapping attempt.
    pub fn as_mut_ionex(&mut self) -> Option<&mut IonexRecord> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }

    /// [MeteoRecord] unwrapping attempt.
    pub fn as_meteo(&self) -> Option<&MeteoRecord> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Mutable [MeteoRecord] unwrapping attempt.
    pub fn as_mut_meteo(&mut self) -> Option<&mut MeteoRecord> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }

    /// [NavRecord] unwrapping attempt.
    pub fn as_nav(&self) -> Option<&NavRecord> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Mutable [NavRecord] unwrapping attempt.
    pub fn as_mut_nav(&mut self) -> Option<&mut NavRecord> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }

    /// [ObservationRecord] unwrapping attempt.
    pub fn as_obs(&self) -> Option<&ObservationRecord> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Mutable [ObservationRecord] unwrapping attempt.
    pub fn as_mut_obs(&mut self) -> Option<&mut ObservationRecord> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }
    /// [DorisRecord] unwrapping attempt.
    pub fn as_doris(&self) -> Option<&DorisRecord> {
        match self {
            Record::DorisRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Mutable [DorisRecord] unwrapping attempt.
    pub fn as_mut_doris(&mut self) -> Option<&mut DorisRecord> {
        match self {
            Record::DorisRecord(r) => Some(r),
            _ => None,
        }
    }
}
