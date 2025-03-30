use qc_traits::QcRepair;

use crate::{
    doris::qc::repair::zero_repair_mut as zero_repair_doris_mut,
    observation::qc::repair::zero_repair_mut as zero_repair_obs_mut, prelude::Rinex,
};

impl QcRepair for Rinex {
    fn zero_repair_mut(&mut self) {
        if let Some(rec) = self.record.as_mut_obs() {
            zero_repair_obs_mut(rec);
        } else if let Some(rec) = self.record.as_mut_doris() {
            zero_repair_doris_mut(rec);
        }
    }
}
