use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    prelude::{Epoch, Header, TimeScale},
};

use qc_traits::QcDecimationFilter;

pub fn decim_mut(header: &mut Header, _: &QcDecimationFilter) {
    header.program = Some(format!(
        "rs-rinex v{}",
        Header::format_pkg_version(env!("CARGO_PKG_VERSION"),)
    ));

    if let Ok(now) = Epoch::now() {
        let now_utc = now.to_time_scale(TimeScale::UTC);
        let (y, m, d, hh, mm, ss, _) = epoch_decomposition(now_utc);
        header.date = Some(format!(
            "{:04}{:02}{:02} {:02}{:02}{:02} UTC",
            y, m, d, hh, mm, ss
        ));
    }
}
