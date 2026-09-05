#[cfg(feature = "ionex")]
#[cfg_attr(docsrs, doc(cfg(feature = "ionex")))]
mod feature;

use crate::{
    ionex::{IonexKey, TEC},
    prelude::{Rinex, RinexType},
};

use std::collections::btree_map::Keys;

impl Rinex {
    /// Returns true if this is a IONEX (special) [Rinex]
    pub fn is_ionex(&self) -> bool {
        self.header.rinex_type == RinexType::IonosphereMaps
    }

    /// Returns true if this IONEX only contains a single isosurface
    /// at fixed altitude. NB: this information only relies
    /// on the [Header] section, not actual data analysis.
    /// If [Record] content did not follow the specifications, this will be invalid.
    pub fn is_ionex_2d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 2 && self.is_ionex()
        } else {
            false
        }
    }

    /// Returns true if this IONEX contains several isosurface spanning [Self::ionex_altitude_range].
    /// NB: this information only relies
    /// on the [Header] section, not actual data analysis.
    /// If [Record] content did not follow the specifications, this will be invalid.
    pub fn is_ionex_3d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 3 && self.is_ionex()
        } else {
            false
        }
    }

    /// Returns [IonexKey] Iterator: the [Epoch](crate::prelude::Epoch) and coordinates
    /// of every [TEC] estimate. This only applies to IONEX and will panic otherwise.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_gzip_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// for key in rinex.ionex_tec_maps_keys() {
    ///     let t = key.epoch;
    ///     let latitude_ddeg = key.coordinates.latitude_ddeg();
    /// }
    /// ```
    pub fn ionex_tec_maps_keys(&self) -> Keys<'_, IonexKey, TEC> {
        if let Some(rec) = self.record.as_ionex() {
            rec.keys()
        } else {
            panic!("invalid operation");
        }
    }

    /// IONEX Total Electron Content Iterator.
    ///
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_gzip_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// for (key, tec) in rinex.ionex_tec_maps_iter() {
    ///     let latitude_ddeg = key.coordinates.latitude_ddeg();
    ///     let longitude_ddeg = key.coordinates.longitude_ddeg();
    ///     let altitude_km = key.coordinates.altitude_km();
    ///     let tec = tec.tec(); // in TEC unit
    /// }
    /// ```
    pub fn ionex_tec_maps_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, &TEC)> + '_> {
        if let Some(rec) = self.record.as_ionex() {
            Box::new(rec.iter().map(|(k, v)| (*k, v)))
        } else {
            Box::new([].into_iter())
        }
    }
}
