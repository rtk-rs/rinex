mod formatting;
pub mod orbits;
mod parsing;

/// Ephemeris NAV flags definitions & support
pub mod flags;

use orbits::OrbitItem;

use flags::{
    bds::{BdsHealth, BdsSatH1},
    gal::GalHealth,
    geo::GeoHealth,
    glonass::{GlonassHealth, GlonassHealth2},
    gps::GpsQzssl1cHealth,
};

#[cfg(feature = "log")]
use log::error;

#[cfg(feature = "nav")]
#[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
mod kepler;

#[cfg(feature = "nav")]
use crate::prelude::nav::Almanac;

use crate::prelude::{Constellation, Duration, Epoch, TimeScale, SV};

#[cfg(feature = "nav")]
use anise::{
    astro::AzElRange,
    errors::AlmanacResult,
    prelude::{Frame, Orbit},
};

use std::collections::HashMap;

/// Ephemeris Navigation message. May be found in all RINEX revisions.
/// Describes the content of the radio message at publication time.
/// Usually published at midnight and regularly updated with respect
/// to [Ephemeris] validity period.
///
/// Any [Ephemeris] comes with the description of the on-board clock,
/// but other data fields are [Constellation] and RINEX version dependent.
/// We store them as dictionary of [OrbitItem]s. This dictionary
/// is parsed based on our built-in JSON descriptor, it proposes methods
/// to access raw data or higher level methods for types that we can interpret.
/// Refer to [OrbitItem] for more information.
///
/// RINEX V3 example:
/// ```
/// use rinex::{
///     prelude::Rinex,
///     navigation::{NavFrameType, NavMessageType},
/// };
///
/// let rinex = Rinex::from_gzip_file("data/NAV/V3/BRDC00GOP_R_20210010000_01D_MN.rnx.gz")
///     .unwrap();
///
/// // You can always unwrap inner structures manually and access everything.
/// // But we propose higher level iteration methods to make things easier:
/// for (key, ephemeris) in rinex.nav_ephemeris_frames_iter() {
///     
///     let sv_broadcaster = key.sv;
///
///     // until RINEXv3 (included) you can only find this type of frame
///     assert_eq!(key.frmtype, NavFrameType::Ephemeris);
///
///     // until RINEXv3 (included) you can only find this kind of message
///     assert_eq!(key.msgtype, NavMessageType::LNAV);
///
///     // Ephemeris serves many purposes and applications, so
///     // it has a lot to offer.
///     
///     if let Some(tgd) = ephemeris.tgd() {
///         // TGD was found & interpreted as duration
///         let tgd = tgd.total_nanoseconds();
///     }
///
///     // SV Health highest interpretation level: as simple boolean
///     if !ephemeris.sv_healthy() {
///         // should most likely be ignored in navigation processing
///     }
///
///     // finer health interpretation is constellation dependent.
///     // Refer to RINEX standards and related constellation ICD.
///     if let Some(health) = ephemeris.orbits.get("health") {
///         if let Some(gps_qzss_l1l2l5) = health.as_gps_qzss_l1l2l5_health_flag() {
///             assert!(gps_qzss_l1l2l5.healthy());
///         }
///     }
/// }
/// ```
///
/// Working with other RINEX revisions does not change anything
/// when dealing with this type, unless maybe the data fields you may
/// find the dictionary. For example, RINEX v4 describes beta-testing
/// health flags for BDS vehicles:
///
/// ```
/// use rinex::{
///     prelude::Rinex,
///     navigation::{NavFrameType, NavMessageType, bds::BdsHealth},
/// };
///
/// let rinex = Rinex::from_gzip_file("data/NAV/V4/BRD400DLR_S_20230710000_01D_MN.rnx.gz")
///     .unwrap();
///
/// // You can always unwrap inner structures manually and access everything.
/// // But we propose higher level iteration methods to make things easier:
/// for (key, ephemeris) in rinex.nav_ephemeris_frames_iter() {
///
///     if let Some(health) = ephemeris.orbits.get("health") {
///         // health flag found & possibly interpreted
///         // this for example, only applies to modern BDS messages
///         if let Some(flag) = health.as_bds_health_flag() {
///             if flag == BdsHealth::UnhealthyTesting {
///             }
///         }
///     }
/// }    
/// ```
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
        let value = self.orbits.get(field)?;
        Some(value.as_f64())
    }

    /// Add a new orbital parameters, encoded as f64.
    pub(crate) fn set_orbit_f64(&mut self, field: &str, value: f64) {
        self.orbits
            .insert(field.to_string(), OrbitItem::from(value));
    }

    /// Try to retrieve week counter. This exists
    /// for all Constellations expect [Constellation::Glonass].
    pub(crate) fn get_week(&self) -> Option<u32> {
        self.orbits
            .get("week")
            .and_then(|value| Some(value.as_u32()))
    }

    /// Returns TGD (if value exists) as [Duration]
    pub fn tgd(&self) -> Option<Duration> {
        let tgd_s = self.get_orbit_f64("tgd")?;
        Some(Duration::from_seconds(tgd_s))
    }

    /// Returns true if this [Ephemeris] declares attached SV as suitable for navigation.
    pub fn sv_healthy(&self) -> bool {
        let health = self.orbits.get("health");

        if health.is_none() {
            return false;
        }

        let health = health.unwrap();

        if let Some(flag) = health.as_gps_qzss_l1l2l5_health_flag() {
            flag.healthy()
        } else if let Some(flag) = health.as_gps_qzss_l1c_health_flag() {
            !flag.intersects(GpsQzssl1cHealth::UNHEALTHY)
        } else if let Some(flag) = health.as_glonass_health_flag() {
            // TODO: Status mask .. ?
            if let Some(flag2) = self
                .orbits
                .get("health2")
                .and_then(|item| Some(item.as_glonass_health2_flag().unwrap()))
            {
                !flag.intersects(GlonassHealth::UNHEALTHY)
                    && flag2.intersects(GlonassHealth2::HEALTHY_ALMANAC)
            } else {
                !flag.intersects(GlonassHealth::UNHEALTHY)
            }
        } else if let Some(flag) = health.as_geo_health_flag() {
            // TODO !
            false
        } else if let Some(flag) = health.as_bds_sat_h1_flag() {
            !flag.intersects(BdsSatH1::UNHEALTHY)
        } else if let Some(flag) = health.as_bds_health_flag() {
            flag == BdsHealth::Healthy
        } else {
            false
        }
    }

    /// Returns true if this [Ephemeris] message declares this satellite in testing mode.
    pub fn sv_in_testing(&self) -> bool {
        let health = self.orbits.get("health");

        if health.is_none() {
            return false;
        }

        let health = health.unwrap();

        // only exists for modern BDS at the moment
        if let Some(flag) = health.as_bds_health_flag() {
            flag == BdsHealth::UnhealthyTesting
        } else {
            false
        }
    }

    /// Returns glonass frequency channel, in case this is a Glonass [Ephemeris] message,
    /// with described channel.
    pub fn glonass_freq_channel(&self) -> Option<i8> {
        if let Some(value) = self.orbits.get("channel") {
            Some(value.as_i8())
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
