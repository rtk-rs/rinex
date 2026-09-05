#[cfg(feature = "nav")]
#[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
mod feature; // feature dependent, high level methods

use crate::{
    navigation::{
        EarthOrientation, Ephemeris, NavFrame, NavFrameType, NavKey, NavMessageType, SystemTime,
    },
    prelude::{Epoch, Rinex, RinexType, SV},
};

use std::collections::btree_map::Keys;

impl Rinex {
    /// Returns true if this [Rinex] is [RinexType::NavigationData].
    /// ```
    /// use rinex::prelude::Rinex;
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// assert!(rinex.is_navigation_rinex());
    /// ```
    pub fn is_navigation_rinex(&self) -> bool {
        self.header.rinex_type == RinexType::NavigationData
    }

    /// Navigation data indexes Iterator
    pub fn navigation_keys(&self) -> Keys<'_, NavKey, NavFrame> {
        if let Some(rec) = self.record.as_nav() {
            rec.keys()
        } else {
            panic!("bad rinex type")
        }
    }

    /// [NavMessageType] Iterator
    pub fn nav_message_types_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, SV, NavMessageType)> + '_> {
        if let Some(rec) = self.record.as_nav() {
            Box::new(rec.iter().map(|(k, _)| (k.epoch, k.sv, k.msgtype)))
        } else {
            Box::new([].into_iter())
        }
    }

    /// [Ephemeris] iter
    pub fn nav_ephemeris_frames_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (&NavKey, &Ephemeris)> + '_> {
        if let Some(rec) = self.record.as_nav() {
            Box::new(rec.iter().filter_map(|(k, v)| {
                if k.frmtype == NavFrameType::Ephemeris {
                    let fr = v.as_ephemeris().unwrap();
                    Some((k, fr))
                } else {
                    None
                }
            }))
        } else {
            Box::new([].into_iter())
        }
    }

    /// [SystemTime] frames iter, which may only exist
    /// in NAV V4 format.
    pub fn nav_system_time_frames_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (&NavKey, &SystemTime)> + '_> {
        if let Some(rec) = self.record.as_nav() {
            Box::new(rec.iter().filter_map(|(k, v)| {
                if k.frmtype == NavFrameType::SystemTimeOffset {
                    let fr = v.as_system_time().unwrap();
                    Some((k, fr))
                } else {
                    None
                }
            }))
        } else {
            Box::new([].into_iter())
        }
    }

    /// [EarthOrientation] frames iter, which may only exist
    /// in NAV V4 format.
    pub fn nav_earth_orientation_frames_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (&NavKey, &EarthOrientation)> + '_> {
        if let Some(rec) = self.record.as_nav() {
            Box::new(rec.iter().filter_map(|(k, v)| {
                if k.frmtype == NavFrameType::EarthOrientation {
                    let fr = v.as_earth_orientation().unwrap();
                    Some((k, fr))
                } else {
                    None
                }
            }))
        } else {
            Box::new([].into_iter())
        }
    }

    /// Returns [SV] clock state Iterator.
    /// ## Inputs
    /// - self: Navigation [Rinex]
    /// ## Output
    /// - offset (s), drift (s.s⁻¹), drift rate (s.s⁻²)  triplet iterator
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// for (key, (offset, drift, drift_rate)) in rinex.nav_sv_clock_iter() {
    ///     // key.epoch: [Epoch] of publication
    ///     // key.sv: satellite vehicle
    ///     // offset [s]
    ///     // clock drift [s.s⁻¹]
    ///     // clock drift rate [s.s⁻²]
    /// }
    /// ```
    pub fn nav_sv_clock_iter(&self) -> Box<dyn Iterator<Item = (NavKey, (f64, f64, f64))> + '_> {
        Box::new(
            self.nav_ephemeris_frames_iter()
                .map(|(k, eph)| (*k, eph.sv_clock())),
        )
    }

    // /*
    //  * [IonMessage] Iterator
    //  */
    // fn ionod_correction_models(
    //     &self,
    // ) -> Box<dyn Iterator<Item = (Epoch, (NavMsgType, SV, IonMessage))> + '_> {
    //     /*
    //      * Answers both OLD and MODERN RINEX requirements
    //      * In RINEX2/3, midnight UTC is the publication datetime
    //      */
    //     let t0 = self.first_epoch().unwrap(); // will fail on invalid RINEX
    //     let t0 = Epoch::from_utc_days(t0.to_utc_days().round());
    //     Box::new(
    //         self.header
    //             .ionod_corrections
    //             .iter()
    //             .map(move |(c, ion)| (t0, (NavMsgType::LNAV, SV::new(*c, 1), *ion)))
    //             .chain(self.navigation().flat_map(|(t, frames)| {
    //                 frames.iter().filter_map(move |fr| {
    //                     let (msg, sv, ion) = fr.as_ion()?;
    //                     Some((*t, (msg, sv, *ion)))
    //                 })
    //             })),
    //     )
    // }
    // /// Returns [`KbModel`] Iterator.
    // /// RINEX4 is the real application of this, as it provides model updates
    // /// during the day. You're probably more interested
    // /// in using [ionod_correction] instead of this, especially in PPP:
    // /// ```
    // /// use rinex::prelude::*;
    // /// use rinex::navigation::KbRegionCode;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, _sv, kb_model) in rinex.klobuchar_models() {
    // ///     let alpha = kb_model.alpha;
    // ///     let beta = kb_model.beta;
    // ///     assert_eq!(kb_model.region, KbRegionCode::WideArea);
    // /// }
    // /// ```
    // /// We support all RINEX3 constellations. When working with this revision,
    // /// you only get one model per day (24 hour validity period). [ionod_correction]
    // /// does that verification internally.
    // /// ```
    // /// use std::str::FromStr;
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    // ///     .unwrap();
    // /// let t0 = Epoch::from_str("2021-01-01T00:00:00 UTC")
    // ///     .unwrap(); // model publication Epoch
    // /// for (t, sv, model) in rinex.klobuchar_models() {
    // ///     assert_eq!(t, t0);
    // ///     // You should use "t==t0" to compare and verify model validity
    // ///     // withint a 24 hour time frame.
    // ///     // Note that we support all RINEX3 constellations
    // ///     if sv.constellation == Constellation::BeiDou {
    // ///         assert_eq!(model.alpha.0, 1.1176E-8);
    // ///     }
    // /// }
    // /// ```
    // /// Klobuchar models exists in RINEX2 and this method applies similarly.
    // pub fn klobuchar_models(&self) -> Box<dyn Iterator<Item = (Epoch, SV, KbModel)> + '_> {
    //     Box::new(
    //         self.ionod_correction_models()
    //             .filter_map(|(t, (_, sv, ion))| ion.as_klobuchar().map(|model| (t, sv, *model))),
    //     )
    // }
    // /// Returns [`NgModel`] Iterator.
    // /// RINEX4 is the real application of this, as it provides model updates
    // /// during the day. You're probably more interested
    // /// in using [ionod_correction] instead of this, especially in PPP:
    // /// ```
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, ng_model) in rinex.nequick_g_models() {
    // ///     let (a0, a1, a2) = ng_model.a;
    // ///     let region = ng_model.region; // bitflag: supports bitmasking operations
    // /// }
    // /// ```
    // /// We support all RINEX3 constellations. When working with this revision,
    // /// you only get one model per day (24 hour validity period). You should prefer
    // /// [ionod_correction] which does that check internally:
    // /// ```
    // /// use std::str::FromStr;
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    // ///     .unwrap();
    // /// let t0 = Epoch::from_str("2021-01-01T00:00:00 UTC")
    // ///     .unwrap(); // model publication Epoch
    // /// for (t, model) in rinex.nequick_g_models() {
    // ///     assert_eq!(t, t0);
    // ///     // You should use "t==t0" to compare and verify model validity
    // ///     // within a 24 hour time frame.
    // ///     assert_eq!(model.a.0, 66.25_f64);
    // /// }
    // /// ```
    // /// Nequick-G model is not known to RINEX2 and only applies to RINEX V>2.
    // pub fn nequick_g_models(&self) -> Box<dyn Iterator<Item = (Epoch, NgModel)> + '_> {
    //     Box::new(
    //         self.ionod_correction_models()
    //             .filter_map(|(t, (_, _, ion))| ion.as_nequick_g().map(|model| (t, *model))),
    //     )
    // }
    // /// Returns [`BdModel`] Iterator.
    // /// RINEX4 is the real application of this, as it provides model updates
    // /// during the day. You're probably more interested
    // /// in using [ionod_correction] instead of this, especially in PPP:
    // /// ```
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, bd_model) in rinex.bdgim_models() {
    // ///     let alpha_tecu = bd_model.alpha;
    // /// }
    // /// ```
    // /// BDGIM was introduced in RINEX4, therefore this method does not apply
    // /// to older revisions and returns an empty Iterator.
    // pub fn bdgim_models(&self) -> Box<dyn Iterator<Item = (Epoch, BdModel)> + '_> {
    //     Box::new(
    //         self.ionod_correction_models()
    //             .filter_map(|(t, (_, _, ion))| ion.as_bdgim().map(|model| (t, *model))),
    //     )
    // }

    // /// Returns Ionospheric delay compensation, to apply at "t" desired Epoch
    // /// and desired location. NB: we only support Klobuchar models at the moment,
    // /// as we don't know how to convert other models (feel free to contribute).
    // /// "t" must be within a 24 hour time frame of the oldest model.
    // /// When working with RINEX2/3, the model is published at midnight
    // /// and you should expect discontinuities when a new model is being published.
    // pub fn ionod_correction(
    //     &self,
    //     t: Epoch,
    //     sv_elevation: f64,
    //     sv_azimuth: f64,
    //     user_lat_ddeg: f64,
    //     user_lon_ddeg: f64,
    //     carrier: Carrier,
    // ) -> Option<f64> {
    //     // determine nearest in time
    //     let (_, (model_sv, model)) = self
    //         .ionod_correction_models()
    //         .filter_map(|(t_i, (_, sv_i, msg_i))| {
    //             // TODO
    //             // calculations currently limited to KB model: implement others
    //             let _ = msg_i.as_klobuchar()?;
    //             // At most 1 day from publication time
    //             if t_i <= t && (t - t_i) < 24.0 * Unit::Hour {
    //                 Some((t_i, (sv_i, msg_i)))
    //             } else {
    //                 None
    //             }
    //         })
    //         .min_by_key(|(t_i, _)| (t - *t_i))?;

    //     // TODO
    //     // calculations currently limited to KB model: implement others
    //     let kb = model.as_klobuchar().unwrap();
    //     let h_km = match model_sv.constellation {
    //         Constellation::BeiDou => 375.0,
    //         // we only expect BDS or GPS here,
    //         // wrongly formed RINEX will cause innacurate results
    //         Constellation::GPS | _ => 350.0,
    //     };
    //     Some(kb.meters_delay(
    //         t,
    //         sv_elevation,
    //         sv_azimuth,
    //         h_km,
    //         user_lat_ddeg,
    //         user_lon_ddeg,
    //         carrier,
    //     ))
    // }

    // /// Returns [`StoMessage`] frames Iterator
    // /// ```
    // /// use rinex::prelude::*;
    // /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, (msg, sv, data)) in rnx.system_time_offset() {
    // ///    let system = data.system.clone(); // time system
    // ///    let utc = data.utc.clone(); // UTC provider
    // ///    let t_tm = data.t_tm; // message transmission time in week seconds
    // ///    let (a, dadt, ddadt) = data.a;
    // /// }
    // /// ```
    // pub fn system_time_offset(
    //     &self,
    // ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, SV, &StoMessage))> + '_> {
    //     Box::new(self.navigation().flat_map(|(e, frames)| {
    //         frames.iter().filter_map(move |fr| {
    //             if let Some((msg, sv, sto)) = fr.as_sto() {
    //                 Some((e, (msg, sv, sto)))
    //             } else {
    //                 None
    //             }
    //         })
    //     }))
    // }

    // /// Returns [`EopMessage`] frames Iterator
    // /// ```
    // /// use rinex::prelude::*;
    // /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, (msg, sv, eop)) in rnx.earth_orientation() {
    // ///     let (x, dxdt, ddxdt) = eop.x;
    // ///     let (y, dydt, ddydt) = eop.x;
    // ///     let t_tm = eop.t_tm;
    // ///     let (u, dudt, ddudt) = eop.delta_ut1;
    // /// }
    // /// ```
    // pub fn earth_orientation(
    //     &self,
    // ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, SV, &EopMessage))> + '_> {
    //     Box::new(self.navigation().flat_map(|(e, frames)| {
    //         frames.iter().filter_map(move |fr| {
    //             if let Some((msg, sv, eop)) = fr.as_eop() {
    //                 Some((e, (msg, sv, eop)))
    //             } else {
    //                 None
    //             }
    //         })
    //     }))
    // }
    // /// Forms a Ut1 Provider as an [DeltaTaiUt1] Iterator from [Self] which must
    // /// be a NAV V4 RINEX file with EOP messages.
    // pub fn ut1_provider(&self) -> Box<dyn Iterator<Item = DeltaTaiUt1> + '_> {
    //     Box::new(
    //         self.earth_orientation()
    //             .map(|(t, (_, _sv, eop))| DeltaTaiUt1 {
    //                 epoch: *t,
    //                 delta_tai_minus_ut1: Duration::from_seconds(eop.delta_ut1.0),
    //             }),
    //     )
    // }
}
