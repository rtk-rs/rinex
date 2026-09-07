use crate::navigation::eph::EphemerisContext;

use std::{cell::RefCell, rc::Rc};

use log::error;

use gnss_rtk::prelude::{BiasRuntime, Duration, SatelliteClockCorrection, SpacebornBias};

/// [ClockContext] implements the solver [SpacebornBias] interface
/// from the ephemeris frames of the [EphemerisContext].
pub struct ClockContext<'a> {
    eph_ctx: Rc<RefCell<EphemerisContext<'a>>>,
}

impl<'a> ClockContext<'a> {
    /// Clock polynomial iterations
    const MAX_ITER: usize = 5;

    pub fn new(eph_ctx: Rc<RefCell<EphemerisContext<'a>>>) -> Self {
        Self { eph_ctx }
    }
}

impl SpacebornBias for ClockContext<'_> {
    fn clock_bias(&self, rtm: &BiasRuntime) -> SatelliteClockCorrection {
        let selected = self.eph_ctx.borrow_mut().select(rtm.epoch, rtm.sv);

        match selected {
            Some((toc, _, eph)) => {
                match eph.clock_correction(toc, rtm.epoch, rtm.sv, Self::MAX_ITER) {
                    Some(dt) => SatelliteClockCorrection::without_relativistic_correction(dt),
                    None => {
                        error!("{}({}): clock correction", rtm.epoch, rtm.sv);
                        Default::default()
                    },
                }
            },
            None => {
                error!("{}({}): ephemeris selection", rtm.epoch, rtm.sv);
                Default::default()
            },
        }
    }

    fn group_delay(&self, rtm: &BiasRuntime) -> Duration {
        let selected = self.eph_ctx.borrow_mut().select(rtm.epoch, rtm.sv);

        match selected {
            Some((_, _, eph)) => eph.tgd().unwrap_or(Duration::ZERO),
            None => Duration::ZERO,
        }
    }

    fn mw_bias(&self, _: &BiasRuntime) -> f64 {
        0.0
    }
}

#[cfg(test)]
mod test {

    use crate::{cfg::QcConfig, context::QcContext};
    use rinex::prelude::{Epoch, SV};
    use std::str::FromStr;

    #[test]
    #[cfg(feature = "flate2")]
    fn test_ephemeris_buffer() {
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(format!(
            "{}/../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

        ctx.load_gzip_file(format!(
            "{}/../test_resources/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

        let mut ctx = ctx.ephemeris_context().expect("ephemeris context failure");

        for (t, sv, exists) in [(
            Epoch::from_str("2020-06-25T04:30:00 GPST").unwrap(),
            SV::from_str("G01").unwrap(),
            true,
        )] {
            if exists {
                let (_toc, _toe, _eph) = ctx.select(t, sv).unwrap();
            }
        }
    }
}
