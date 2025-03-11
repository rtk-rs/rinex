//! Feature dependent high level methods

use crate::{
    meteo::Sensor,
    prelude::{Epoch, Observable, Rinex},
};

impl Rinex {
    /// Returns Meteo [Sensor]s Iterator
    pub fn meteo_sensors_iter(&self) -> Box<dyn Iterator<Item = &Sensor> + '_> {
        if let Some(meteo) = &self.header.meteo {
            Box::new(meteo.sensors.iter())
        } else {
            Box::new([].into_iter())
        }
    }

    /// Returns wind speed estimates iterator, values expressed in m/s.
    /// Applies to Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.wind_speed_iter() {
    ///     println!("{} value: {} m/s", epoch, value);
    /// }
    /// ```
    pub fn wind_speed_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
            if k.observable == Observable::WindSpeed {
                Some((k.epoch, *v))
            } else {
                None
            }
        }))
    }

    /// Returns wind direction estimates iterator, values expressed as azimuth degrees.
    /// Applies to Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.wind_direction_iter() {
    ///     println!("{} value: {}°", epoch, value);
    /// }
    /// ```
    pub fn wind_direction_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
            if k.observable == Observable::WindDirection {
                Some((k.epoch, *v))
            } else {
                None
            }
        }))
    }

    /// Returns accumulated rain iterator (between successive [Epoch]s).
    /// Values expressed in tenths of mm. Applies to Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.accumulated_rain_iter() {
    ///     println!("{} value: {} 1/10 mm", epoch, value);
    /// }
    /// ```
    pub fn accumulated_rain_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
            if k.observable == Observable::RainIncrement {
                Some((k.epoch, *v))
            } else {
                None
            }
        }))
    }

    /// Returns total rain height accumulated within covered time frame.
    /// Applies to Meteo RINEX.
    pub fn total_accumulated_rain(&self) -> f64 {
        self.accumulated_rain_iter().fold(0.0, |mut acc, k_v| {
            acc = acc + k_v.1;
            acc
        })
    }

    /// Returns true if rain dropped during the observed time frame.
    /// Applies to Meteo RINEX.
    pub fn rain_detected(&self) -> bool {
        self.total_accumulated_rain() > 0.0
    }

    /// Returns true if hail dropped during the observed time frame.
    /// Applies to Meteo RINEX.
    pub fn hail_detected(&self) -> bool {
        for (k, v) in self.meteo_observations_iter() {
            if k.observable == Observable::HailIndicator {
                if *v > 0.0 {
                    return true;
                }
            }
        }
        false
    }

    /// Returns total Wet + Dry Zenith delay components, in mm.
    /// Applies to Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_total_delay_iter() {
    ///     println!("{} value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_total_delay_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
            if k.observable == Observable::ZenithTotalDelay {
                Some((k.epoch, *v))
            } else {
                None
            }
        }))
    }

    /// Returns Zenith Dry delay components, in mm.
    /// Applies to Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_dry_delay_iter() {
    ///     println!("{} value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_dry_delay_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
            if k.observable == Observable::ZenithDryDelay {
                Some((k.epoch, *v))
            } else {
                None
            }
        }))
    }

    /// Returns Zenith Wet delay components, in mm.
    /// Applies to Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_wet_delay_iter() {
    ///     println!("{} value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_wet_delay_iter(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo_observations_iter().filter_map(|(k, v)| {
            if k.observable == Observable::ZenithWetDelay {
                Some((k.epoch, *v))
            } else {
                None
            }
        }))
    }

    //   /// Returns true if hail was detected during this time frame
    //   /// ```
    //   /// use std::str::FromStr;
    //   /// use rinex::prelude::*;
    //   /// use rinex::prelude::Preprocessing; // only on "processing" feature
    //   ///
    //   /// // parse a RINEX
    //   /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    //   ///     .unwrap();
    //   ///
    //   /// // only when built with "processing" feature
    //   /// let morning = Filter::lower_than("2015-01-01T12:00:00 UTC")
    //   ///     .unwrap();
    //   ///
    //   /// let rinex = rinex.filter(&morning);
    //   /// assert_eq!(rinex.accumulated_rain(), 0.0);
    //   /// assert_eq!(rinex.rain_detected(), false);
    //   /// ```
    //   pub fn hail_detected(&self) -> bool {
    //       if let Some(r) = self.record.as_meteo() {
    //           for observables in r.values() {
    //               for (observ, value) in observables {
    //                   if *observ == Observable::HailIndicator && *value > 0.0 {
    //                       return true;
    //                   }
    //               }
    //           }
    //           false
    //       } else {
    //           false
    //       }
    //   }
} //
