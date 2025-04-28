use qc_traits::GnssAbsoluteTime;

use crate::prelude::Rinex;

impl Rinex {
    /// Collect [GnssAbsoluteTime] solver from this Navigation [Rinex].
    /// Does not apply to any other format.
    pub fn gnss_absolute_time_solver(&self) -> Option<GnssAbsoluteTime> {
        if !self.is_navigation_rinex() {
            return None;
        }

        let mut polynomials = Vec::new();

        // collect from possible V3 header
        let header = self.header.nav.as_ref()?;

        for value in header.time_offsets.iter() {
            polynomials.push(value.to_time_polynomial());
        }

        // collect from possible V4 frames
        // TODO: don't know how to interprate the ref time
        let record = self.record.as_nav()?;

        Some(GnssAbsoluteTime::new(&polynomials))
    }
}
