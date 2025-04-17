use crate::prelude::{Rinex, Epoch, TimeScale, Header};
use qc_traits::{Timeshift GnssAbsoluteTime};

impl Timeshift for Header {
    fn time_shift(&self, target: TimeScale, solver: &GnssAbsoluteTime) -> Self {
        let mut s = self.clone();
        s.time_shift_mut(target, &solver);
        s
    }

    fn time_shift_mut(&mut self, target: TimeScale, solver: &GnssAbsoluteTime) {
        if let Some(obs) = &mut self.obs {
            if let Some(epoch) = &mut obs.time_of_first_obs {
                *epoch = solver.epoch_time_correction(epoch, target);
            }
            if let Some(epoch) = &mut obs.time_of_last_obs {
                *epoch = solver.epoch_time_correction(epoch, target);
            }
        }
    }
}

impl Timeshift for Rinex {
    fn time_shift(&self, target: TimeScale, solver: &GnssAbsoluteTime) -> Self {
        let mut s = self.clone();
        s.time_shift_mut(target, &solver);
        s
    }

    fn time_shift_mut(&mut self, target: TimeScale, solver: &GnssAbsoluteTime) {
        self.header.timeshift_mut(target, solver);

        if let Some(obs) = self.record.as_mut_obs() {
            for (k, _) in obs.iter_mut() {
                *k.epoch = solver.epoch_time_correction(k.epoch, target);
            }
        }
    }
}
