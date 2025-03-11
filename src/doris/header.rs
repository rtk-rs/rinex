//! DORIS special Header fields
//!
use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};

use crate::{
    doris::Station,
    fmt_rinex,
    prelude::{Duration, Epoch, FormattingError, Observable},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "processing")]
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

/// DORIS specific header fields
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Name of the DORIS satellite
    pub satellite: String,
    /// Time of First Measurement, expressed in TAI timescale.
    pub timeof_first_obs: Option<Epoch>,
    /// Time of Last Measurement, expressed in TAI timescale.
    pub timeof_last_obs: Option<Epoch>,
    /// List of observables
    pub observables: Vec<Observable>,
    /// Data scaling, almost 100% of the time present in DORIS measurements.
    /// Allows precision down to 1E-9 radians on signal phase demodulation.
    pub scaling: HashMap<Observable, u16>,
    /// Reference stations present in this file
    pub stations: Vec<Station>,
    /// Constant offset between timestamp of the U2 (401.25 MHz) phase measurement
    /// and S1 (2.03625 GHz) phase measurement
    pub u2_s1_time_offset: Duration,
}

impl HeaderFields {
    /// Formats [HeaderFields] into [BufWriter].
    pub(crate) fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        write!(w, "{}", fmt_rinex(&self.satellite, "SATELLITE NAME"))?;
        write!(w, "{}", fmt_rinex(&self.satellite, "SATELLITE NAME"))?;

        let l2_l1_date_offset = format!("D     {:10.3}", self.u2_s1_time_offset.to_seconds());
        write!(
            w,
            "{}",
            fmt_rinex(&l2_l1_date_offset, "L2 / L1 DATE OFFSET")
        )?;

        let num_stations = format!("{:<10}", self.stations.len());
        write!(w, "{}", fmt_rinex(&num_stations, "# OF STATIONS"))?;

        for station in self.stations.iter() {
            write!(
                w,
                "{}",
                fmt_rinex(&station.to_string(), "STATION REFERENCE")
            )?;
        }

        Ok(())
    }

    // /// Retrieve station by ID#
    // pub(crate) fn get_station(&mut self, id: u16) -> Option<&Station> {
    //     self.stations
    //         .iter()
    //         .filter(|s| s.key == id)
    //         .reduce(|k, _| k)
    // }

    /// Insert a data scaling
    pub(crate) fn with_scaling(&mut self, observable: Observable, scaling: u16) {
        self.scaling.insert(observable.clone(), scaling);
    }

    // /// Returns scaling to applied to said Observable.
    // pub(crate) fn scaling(&self, observable: Observable) -> Option<&u16> {
    //     self.scaling.get(&observable)
    // }
}

#[cfg(feature = "processing")]
impl HeaderFields {
    // fn timescale(&self) -> TimeScale {
    //     match self.timeof_first_obs {
    //         Some(ts) => ts.time_scale,
    //         None => match self.timeof_last_obs {
    //             Some(ts) => ts.time_scale,
    //             None => TimeScale::GPST,
    //         },
    //     }
    // }

    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(_epoch) => {},
                _ => {},
            },
            MaskOperand::NotEquals => match &f.item {
                FilterItem::EpochItem(_epoch) => {},
                _ => {},
            },
            MaskOperand::GreaterThan => match &f.item {
                FilterItem::EpochItem(_epoch) => {},
                _ => {},
            },
            MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(_epoch) => {},
                _ => {},
            },
            MaskOperand::LowerThan => match &f.item {
                FilterItem::EpochItem(_epoch) => {},
                _ => {},
            },
            MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(_epoch) => {},
                _ => {},
            },
        }
    }
}
