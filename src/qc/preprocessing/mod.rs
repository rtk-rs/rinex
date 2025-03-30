use qc_traits::{QcFilter, QcPreprocessing};

use crate::prelude::Rinex;

#[cfg(feature = "qc")]
use crate::{
    clock::record::clock_mask_mut, doris::mask::mask_mut as doris_mask_mut,
    header::processing::header_mask_mut, ionex::mask_mut as ionex_mask_mut,
    meteo::mask::mask_mut as meteo_mask_mut, navigation::mask::mask_mut as navigation_mask_mut,
    observation::mask::mask_mut as observation_mask_mut,
};

impl QcPreprocessing for Rinex {
    fn filter_mut(&mut self, f: &QcFilter) {
        self.header.filter_mut(f);

        if let Some(rec) = self.record.as_mut_obs() {
            observation_filter_mut(rec);
        } else if let Some(rec) = self.record.as_mut_nav() {
            navigation_filter_mut(rec);
        } else if let Some(rec) = self.record.as_mut_clock() {
            clock_filter_mut(rec);
        } else if let Some(rec) = self.record.as_mut_ionex() {
            ionex_filter_mut(rec);
        } else if let Some(rec) = self.record.as_mut_meteo() {
            meteo_filter_mut(rec);
        } else if let Some(rec) = self.record.as_mut_doris() {
            doris_filter_mut(rec);
        }

        self.production.filter_mut(f);
    }
}
