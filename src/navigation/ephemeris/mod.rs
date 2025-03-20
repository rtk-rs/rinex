mod formatting;
mod parsing;

#[cfg(feature = "log")]
use log::error;

#[cfg(feature = "nav")]
#[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
mod kepler;

#[cfg(feature = "nav")]
use crate::prelude::nav::Almanac;

use crate::{
    navigation::OrbitItem,
    prelude::{Constellation, Duration, Epoch, TimeScale, SV},
};
#[cfg(feature = "nav")]
use anise::{
    astro::AzElRange,
    errors::AlmanacResult,
    prelude::{Frame, Orbit},
};

use std::collections::HashMap;

/// Ephermeris NAV frame type
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Ephemeris {
    /// Clock bias (in seconds)
    pub clock_bias: f64,
    /// Clock drift (s.s⁻¹)
    pub clock_drift: f64,
    /// Clock drift rate (s.s⁻²)).   
    pub clock_drift_rate: f64,
    /// Orbits are revision and constellation dependent,
    /// sorted by key and content, described in navigation::database
    pub orbits: HashMap<String, OrbitItem>,
}

impl Ephemeris {
    /// Returns [SV] onboard clock (bias [s], drift [s/s], drift rate [s/s]).
    pub fn sv_clock(&self) -> (f64, f64, f64) {
        (self.clock_bias, self.clock_drift, self.clock_drift_rate)
    }

    /// Returns abstract orbital parameter from readable description and
    /// interprated as f64.
    pub fn get_orbit_f64(&self, field: &str) -> Option<f64> {
        if let Some(value) = self.orbits.get(field) {
            let value = value.as_f64()?;
            if value != 0.0 {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Add a new orbital parameters, encoded as f64.
    pub(crate) fn set_orbit_f64(&mut self, field: &str, value: f64) {
        self.orbits
            .insert(field.to_string(), OrbitItem::from(value));
    }

    /// Try to retrieve week counter. This exists
    /// for all Constellations expect [Constellation::Glonass].
    pub(crate) fn get_week(&self) -> Option<u32> {
        self.orbits.get("week").and_then(|value| value.as_u32())
    }

    /// Returns TGD (if value exists) as [Duration]
    pub fn tgd(&self) -> Option<Duration> {
        let tgd_s = self.get_orbit_f64("tgd")?;
        Some(Duration::from_seconds(tgd_s))
    }

    /// Returns glonass frequency channel, in case this is a Glonass [Ephemeris] message,
    /// with described channel.
    pub fn glonass_freq_channel(&self) -> Option<i8> {
        if let Some(value) = self.orbits.get("channel") {
            let value = value.as_i8()?;
            Some(value)
        } else {
            None
        }
    }

    /// Return ToE expressed as [Epoch]
    pub fn toe(&self, sv_ts: TimeScale) -> Option<Epoch> {
        // TODO: in CNAV V4 TOC is said to be TOE... ...
        let week = self.get_week()?;
        let sec = self.get_orbit_f64("toe")?;
        let week_dur = Duration::from_days((week * 7) as f64);
        let sec_dur = Duration::from_seconds(sec);
        match sv_ts {
            TimeScale::GPST | TimeScale::QZSST | TimeScale::GST => {
                Some(Epoch::from_duration(week_dur + sec_dur, TimeScale::GPST))
            },
            TimeScale::BDT => Some(Epoch::from_bdt_duration(week_dur + sec_dur)),
            _ => {
                #[cfg(feature = "log")]
                error!("{} is not supported", sv_ts);
                None
            },
        }
    }

    /// Returns Adot parameter from a CNAV ephemeris
    pub(crate) fn a_dot(&self) -> Option<f64> {
        self.get_orbit_f64("a_dot")
    }
}

impl Ephemeris {
    /// Creates new [Ephemeris] with desired [OrbitItem]
    pub fn with_orbit(&self, key: &str, orbit: OrbitItem) -> Self {
        let mut s = self.clone();
        s.orbits.insert(key.to_string(), orbit);
        s
    }

    /// Creates new [Ephemeris] with desired week counter
    pub fn with_week(&self, week: u32) -> Self {
        self.with_orbit("week", OrbitItem::from(week))
    }

    /// Calculates Clock correction for [SV] at [Epoch] based on [Self]
    /// and ToC [Epoch] of publication of [Self] from the free running clock.
    pub fn clock_correction(
        &self,
        toc: Epoch,
        t: Epoch,
        sv: SV,
        max_iter: usize,
    ) -> Option<Duration> {
        let sv_ts = sv.constellation.timescale()?;

        let t_sv = t.to_time_scale(sv_ts);
        let toc_sv = toc.to_time_scale(sv_ts);

        if t_sv < toc_sv {
            #[cfg(feature = "log")]
            error!("t < t_oc: bad op!");
            None
        } else {
            let (a0, a1, a2) = (self.clock_bias, self.clock_drift, self.clock_drift_rate);
            let mut dt = (t_sv - toc_sv).to_seconds();
            for _ in 0..max_iter {
                dt -= a0 + a1 * dt + a2 * dt.powi(2);
            }
            Some(Duration::from_seconds(a0 + a1 * dt + a2 * dt.powi(2)))
        }
    }

    /// (elevation, azimuth, range) determination helper,
    /// returned in the form of [AzElRange], for desired [SV] observed at RX coordinates,
    /// expressed in km in fixed body [Frame] centered on Earth.
    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    pub fn elevation_azimuth_range(
        t: Epoch,
        almanac: &Almanac,
        fixed_body_frame: Frame,
        sv_position_km: (f64, f64, f64),
        rx_position_km: (f64, f64, f64),
    ) -> AlmanacResult<AzElRange> {
        let (rx_x_km, rx_y_km, rx_z_km) = rx_position_km;
        let (tx_x_km, tx_y_km, tx_z_km) = sv_position_km;

        let rx_orbit = Orbit::from_position(rx_x_km, rx_y_km, rx_z_km, t, fixed_body_frame);
        let tx_orbit = Orbit::from_position(tx_x_km, tx_y_km, tx_z_km, t, fixed_body_frame);

        almanac.azimuth_elevation_range_sez(rx_orbit, tx_orbit, None, None)
    }

    /// Returns True if Self is Valid at specified `t`.
    /// NB: this only applies to MEO Ephemerides, not GEO Ephemerides,
    /// which should always be considered "valid".
    pub fn is_valid(&self, sv: SV, t: Epoch, toe: Epoch) -> bool {
        if let Some(max_dtoe) = Self::validity_duration(sv.constellation) {
            t > toe && (t - toe) < max_dtoe
        } else {
            #[cfg(feature = "log")]
            error!("{} not fully supported", sv.constellation);
            false
        }
    }

    /// Ephemeris validity period for this [Constellation]
    pub fn validity_duration(c: Constellation) -> Option<Duration> {
        match c {
            Constellation::GPS | Constellation::QZSS => Some(Duration::from_seconds(7200.0)),
            Constellation::Galileo => Some(Duration::from_seconds(10800.0)),
            Constellation::BeiDou => Some(Duration::from_seconds(21600.0)),
            Constellation::IRNSS => Some(Duration::from_seconds(7200.0)),
            Constellation::Glonass => Some(Duration::from_seconds(1800.0)),
            c => {
                if c.is_sbas() {
                    // Tolerate one publication per day.
                    // Typical RINEX apps will load one set per 24 hr.
                    // GEO Orbits are very special, with a single entry per day.
                    // Therefore, in typical RINEX apps, we will have one entry for every day.
                    // GEO Ephemerides cannot be handled like other Ephemerides anyway, they require
                    // a complete different logic and calculations
                    Some(Duration::from_days(1.0))
                } else {
                    None
                }
            },
        }
    }
}
