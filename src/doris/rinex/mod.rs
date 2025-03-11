#[cfg(feature = "doris")]
#[cfg_attr(docsrs, doc(cfg(feature = "doris")))]
mod feature;

use std::collections::btree_map::{Iter, IterMut, Keys};

use crate::{
    doris::{DorisKey, Observations, SignalKey, SignalObservation},
    observation::ClockObservation,
    prelude::{Rinex, RinexType},
};

impl Rinex {
    /// Returns true if this is DORIS special RINEX
    pub fn is_doris(&self) -> bool {
        self.header.rinex_type == RinexType::DORIS
    }

    pub fn doris_observation_keys(&self) -> Keys<'_, DorisKey, Observations> {
        if let Some(rec) = self.record.as_doris() {
            rec.keys()
        } else {
            panic!("invalid");
        }
    }

    pub fn doris_observation_iter(&self) -> Iter<'_, DorisKey, Observations> {
        if let Some(rec) = self.record.as_doris() {
            rec.iter()
        } else {
            panic!("invalid");
        }
    }

    pub fn doris_observation_iter_mut(&mut self) -> IterMut<'_, DorisKey, Observations> {
        if let Some(rec) = self.record.as_mut_doris() {
            rec.iter_mut()
        } else {
            panic!("invalid");
        }
    }

    /// Returns DORIS satellite (onboard) clock offset iterator.
    /// Use [HeaderFields.satellite] to determine which DORIS satellite we're talking about:
    /// one DORIS satellite per file. Use [DorisObservation.clock_extrapolated] to determine
    /// whether this is an extrapolation or actual measurement.
    /// Offset is offset to TAI timescale expressed as [Duration].
    pub fn doris_satellite_clock_offset_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (DorisKey, ClockObservation)> + '_> {
        Box::new([].into_iter())
    }

    /// Returns Iterator over all Ground [Station] observations, made by
    /// this DORIS satellite. Use [HeaderFields.satellite] to determine which DORIS satellite we're talking about:
    /// one DORIS satellite per file. Use [Observable] to determine the signal, physics and signal modulation.
    pub fn doris_ground_station_signals_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (DorisKey, SignalKey, SignalObservation)> + '_> {
        Box::new([].into_iter())
    }
}
