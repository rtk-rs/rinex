use crate::{
    doris::{DorisKey, Station},
    prelude::Rinex,
};

impl Rinex {
    /// Returns DORIS Ground [Station]s Iterator
    pub fn doris_ground_stations_iter(&self) -> Box<dyn Iterator<Item = &Station> + '_> {
        if let Some(doris) = &self.header.doris {
            Box::new(doris.stations.iter())
        } else {
            Box::new([].into_iter())
        }
    }

    /// Returns DORIS satellite (onboard) clock drift iterator.
    /// Use [HeaderFields.satellite] to determine which DORIS satellite we're talking about:
    /// one DORIS satellite per file. Use [DorisObservation.clock_extrapolated] to determine
    /// whether this is an extrapolation or actual measurement.
    /// Drift is expressed in TAI timescale in seconds per second.
    pub fn doris_satellite_clock_drift_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (DorisKey, f64)> + '_> {
        Box::new([].into_iter())
    }

    /// Returns Iterator over all pseudo range observations from all ground stations, expressed in meters.
    pub fn doris_ground_station_pseudo_range_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (DorisKey, f64)> + '_> {
        Box::new([].into_iter())
    }

    /// Returns Iterator over all phase range observations from all ground stations, expressed in meters.
    pub fn doris_ground_station_phase_range_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (DorisKey, f64)> + '_> {
        Box::new([].into_iter())
    }
}
