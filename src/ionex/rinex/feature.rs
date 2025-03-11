use crate::{
    ionex::{IonexKey, TEC},
    prelude::{Epoch, Rinex},
};

impl Rinex {
    /// Returns Iterator over TEC expressed in TECu (10^-16 m-2) per lattitude,
    /// longitude in decimal degrees, and altitude in km.
    pub fn ionex_tecu_latlong_ddeg_alt_km_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, f64, f64, f64, f64)> + '_> {
        Box::new(self.ionex_tec_maps_iter().map(|(k, v)| {
            (
                k.epoch,
                k.coordinates.latitude_ddeg(),
                k.coordinates.longitude_ddeg(),
                k.coordinates.altitude_km(),
                v.tecu(),
            )
        }))
    }

    /// Returns IONEX TEC map borders.
    /// NB: this is only based on Header definitions. If provided
    /// content did not follow those specs (incorrect), the returned value here will not
    /// reflect actual content.
    /// ## Output
    /// - (lat_min, lat_max): southernmost and northernmost latitude, in decimal degrees
    /// - (long_min, long_max): easternmost and westernmost longitude, in decimal degrees
    pub fn tec_map_borders(&self) -> Option<((f64, f64), (f64, f64))> {
        let ionex = self.header.ionex.as_ref()?;
        Some((
            (ionex.grid.latitude.start, ionex.grid.latitude.end),
            (ionex.grid.longitude.start, ionex.grid.longitude.end),
        ))
    }

    /// Designs an iterator over RMS TEC exclusively
    pub fn ionex_rms_tec_maps_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, f64)> + '_> {
        Box::new(self.ionex_tec_maps_iter().filter_map(|(k, tec)| {
            if let Some(rms) = tec.rms_tec() {
                Some((k, rms))
            } else {
                None
            }
        }))
    }

    /// Returns fixed altitude (in kilometers) if this is a 2D IONEX (only).
    /// Note that this is only based on Header definitions. If provided
    /// content does not follow those specs (incorrect data), the returned value here will not
    /// reflect actual content.
    pub fn ionex_2d_fixed_altitude_km(&self) -> Option<f64> {
        if self.is_ionex_2d() {
            let header = self.header.ionex.as_ref()?;
            Some(header.grid.height.start)
        } else {
            None
        }
    }

    /// Returns altitude range of the IONEX TEC maps, expressed as {min, max}
    /// both in kilometers.
    /// - if this is a 2D IONEX: you will obtain (min, min)
    /// - if this is a 3D IONEX: you will obtain (min, max)
    /// Note that this is only based on Header definitions. If provided
    /// content does not follow those specs (incorrect data), the returned value here will not
    /// reflect actual content.
    pub fn ionex_altitude_range_km(&self) -> Option<(f64, f64)> {
        let header = self.header.ionex.as_ref()?;
        Some((header.grid.height.start, header.grid.height.end))
    }

    /// Designs an TEC isosurface iterator starting at lowest altitude,
    /// ending at highest altitude. See [Self::ionex_tec_maps_altitude_range] for
    /// useful information.
    pub fn ionex_tec_isosurface_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, TEC)> + '_> {
        Box::new([].into_iter())
    }

    /// Designs an RMS TEC isosurface iterator starting at lowest altitude,
    /// ending at highest altitude. See [Self::ionex_tec_maps_altitude_range] for
    /// useful information.
    pub fn ionex_rms_tec_isosurface_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, TEC)> + '_> {
        Box::new([].into_iter())
    }
}
