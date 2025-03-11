use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    prelude::{Epoch, Header, TimeScale},
};

use qc_traits::{Decimate, DecimationFilter};

impl Decimate for Header {
    fn decimate(&self, f: &DecimationFilter) -> Self {
        let mut s = self.clone();
        s.decimate_mut(f);
        s
    }
    fn decimate_mut(&mut self, _: &DecimationFilter) {
        self.program = Some(format!(
            "rs-rinex v{}",
            Self::format_pkg_version(env!("CARGO_PKG_VERSION"),)
        ));

        if let Ok(now) = Epoch::now() {
            let now_utc = now.to_time_scale(TimeScale::UTC);
            let (y, m, d, hh, mm, ss, _) = epoch_decomposition(now_utc);
            self.date = Some(format!(
                "{:04}{:02}{:02} {:02}{:02}{:02} UTC",
                y, m, d, hh, mm, ss
            ));
        }
    }
}
