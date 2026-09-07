use rinex::{
    navigation::Ephemeris,
    prelude::{Epoch, SV},
};

use crate::context::QcContext;

use gnss_rtk::prelude::{Ephemeris as RTKEphemeris, EphemerisSource};

use std::collections::HashMap;

use log::error;

pub struct EphemerisContext<'a> {
    eos: bool,
    buffer: HashMap<SV, Vec<(Epoch, Epoch, Ephemeris)>>,
    iter: Box<dyn Iterator<Item = (SV, Epoch, &'a Ephemeris)> + 'a>,
}

impl<'a> EphemerisContext<'a> {
    /// Bufferize content until ephemeris is valid for [SV] at [Epoch]
    fn bufferize_until_valid(&mut self, sel_t: Epoch, sel_sv: SV) {
        // timescale needs to be determined
        let sel_ts = sel_sv.constellation.timescale();
        if sel_ts.is_none() {
            return;
        }

        let sel_ts = sel_ts.unwrap();

        loop {
            if let Some((sv, toc, eph)) = self.iter.next() {
                // calculate ToE
                if let Some(toe) = eph.toe(sel_ts) {
                    // store
                    if let Some(data) = self.buffer.get_mut(&sv) {
                        data.push((toc, toe, eph.clone()));
                    } else {
                        // new
                        self.buffer.insert(sv, vec![(toc, toe, eph.clone())]);
                    }

                    // exit(stop bffering) when validity is obtained for target.
                    if sel_sv == sv {
                        if !eph.is_valid(sv, sel_t, toe) {
                            break;
                        }
                    }
                } else {
                    error!("{}({}): toe", sel_t, sel_sv);
                }
            } else {
                self.eos = true;
                break;
            }
        }
    }

    /// Select closest bufferized [Ephemeris] & compute ToE at the same time.
    /// NB: ToE does not exist for SBAS, we simply copy ToC to maintain the API.
    fn closest_in_time(&self, t: Epoch, sv: SV) -> Option<(Epoch, Epoch, &Ephemeris)> {
        let buffered = self.buffer.get(&sv)?;

        if sv.constellation.is_sbas() {
            buffered
                .iter()
                .filter_map(|(toc, toe, eph)| {
                    if t >= *toc {
                        Some((*toc, *toe, eph))
                    } else {
                        None
                    }
                })
                .map(|(toc, toe, eph)| (toc, toe, eph))
                .min_by_key(|(toc_i, _, _)| (t - *toc_i).abs())
        } else {
            buffered
                .iter()
                .filter_map(|(toc, toe, eph)| {
                    if t >= *toc {
                        Some((*toc, *toe, eph))
                    } else {
                        None
                    }
                })
                .map(|(toc, toe, eph)| (toc, toe, eph))
                .min_by_key(|(toc_i, _, _)| (t - *toc_i).abs())
        }
    }

    /// Select appropriate [Ephemeris] for navigation purposes
    /// ## Input
    /// - t: desired [Epoch]
    /// - sv: desired [SV]
    /// ## Output
    /// - toc: [Epoch]
    /// - toe: [Epoch]
    /// - eph: [Ephemeris]
    pub fn select(&mut self, t: Epoch, sv: SV) -> Option<(Epoch, Epoch, Ephemeris)> {
        if !self.eos {
            self.bufferize_until_valid(t, sv);
        }

        let (toc, toe, eph) = self.closest_in_time(t, sv)?;
        Some((toc, toe, eph.clone()))
    }
}

/// [NullEphemeris] implements the solver [EphemerisSource] interface,
/// which the solver does not use yet: orbital states are provided
/// by the [OrbitSource] implementation instead.
pub struct NullEphemeris {}

impl EphemerisSource for NullEphemeris {
    fn ephemeris_data(&self, _: Epoch, _: SV) -> Option<RTKEphemeris> {
        None
    }
}

impl QcContext {
    /// Try to obtain an [EphemerisContext] ready to browse all Ephemeris contained
    /// the in data set.
    pub fn ephemeris_context<'a>(&'a self) -> Option<EphemerisContext<'a>> {
        if !self.has_navigation_data() {
            return None;
        }

        let nav_dataset = self.nav_dataset.as_ref().unwrap();

        Some(EphemerisContext {
            eos: false,
            buffer: HashMap::with_capacity(8),
            iter: Box::new(
                nav_dataset
                    .nav_ephemeris_frames_iter()
                    .map(|(k, v)| (k.sv, k.epoch, v)),
            ),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::{cfg::QcConfig, context::QcContext};
    use rinex::prelude::{Epoch, SV};
    use std::str::FromStr;

    #[test]
    #[cfg(feature = "flate2")]
    fn bufferized_ephemeris() {
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(format!(
            "{}/../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

        let mut ctx = ctx.ephemeris_context().expect("ephemeris context failure");

        for (t, sv, toc) in [
            (
                Epoch::from_str("2020-06-24T22:10:00 GPST").unwrap(),
                SV::from_str("G30").unwrap(),
                Some(Epoch::from_str("2020-06-24T22:00:00 GPST").unwrap()),
            ),
            (
                Epoch::from_str("2020-06-24T23:30:00 GPST").unwrap(),
                SV::from_str("G08").unwrap(),
                None,
            ),
            (
                Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap(),
                SV::from_str("G08").unwrap(),
                Some(Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap()),
            ),
            (
                Epoch::from_str("2020-06-25T00:01:00 GPST").unwrap(),
                SV::from_str("G08").unwrap(),
                Some(Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap()),
            ),
            (
                Epoch::from_str("2020-06-25T01:59:00 GPST").unwrap(),
                SV::from_str("G08").unwrap(),
                Some(Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap()),
            ),
            (
                Epoch::from_str("2020-06-25T01:59:44 GPST").unwrap(),
                SV::from_str("G08").unwrap(),
                Some(Epoch::from_str("2020-06-25T01:59:44 GPST").unwrap()),
            ),
            (
                Epoch::from_str("2020-06-25T01:59:45 GPST").unwrap(),
                SV::from_str("G08").unwrap(),
                Some(Epoch::from_str("2020-06-25T01:59:44 GPST").unwrap()),
            ),
        ] {
            if let Some(toc) = toc {
                let (toc_i, _, _) = ctx.select(t, sv).unwrap_or_else(|| {
                    let buffered = ctx.buffer.get(&sv).unwrap();
                    panic!("missed selection for {}({}): {:#?}", t, sv, buffered);
                });

                assert_eq!(toc_i, toc);
            } else {
                assert!(
                    ctx.select(t, sv).is_none(),
                    "invalid selection {}({})",
                    t,
                    sv
                );
            }
        }
    }
}
