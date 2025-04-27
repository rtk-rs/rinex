use crate::{
    observation::{ObsKey, Observations},
    prelude::{Duration, Header, Rinex, TimeScale},
};
use qc_traits::{GnssAbsoluteTime, Timeshift};

use std::collections::BTreeMap;

impl Timeshift for Header {
    fn timeshift(&self, solver: &GnssAbsoluteTime, target: TimeScale) -> Self
    where
        Self: Sized,
    {
        let mut s = self.clone();
        s.timeshift_mut(solver, target);
        s
    }

    fn timeshift_mut(&mut self, solver: &GnssAbsoluteTime, target: TimeScale) {
        if let Some(obs) = &mut self.obs {
            if let Some(epoch) = &mut obs.timeof_first_obs {
                if let Some(converted) = solver.precise_epoch_correction(*epoch, target) {
                    *epoch = converted.round(Duration::from_microseconds(1.0));
                } else {
                    obs.timeof_first_obs = None;
                }
            }

            if let Some(epoch) = &mut obs.timeof_last_obs {
                if let Some(converted) = solver.precise_epoch_correction(*epoch, target) {
                    *epoch = converted.round(Duration::from_microseconds(1.0));
                } else {
                    obs.timeof_last_obs = None;
                }
            }
        }
    }
}

impl Timeshift for Rinex {
    fn timeshift(&self, solver: &GnssAbsoluteTime, target: TimeScale) -> Self
    where
        Self: Sized,
    {
        let mut s = self.clone();
        s.timeshift_mut(solver, target);
        s
    }

    fn timeshift_mut(&mut self, solver: &GnssAbsoluteTime, target: TimeScale) {
        self.header.timeshift_mut(solver, target);

        if let Some(obs_rec) = self.record.as_mut_obs() {
            let mut new_rec = BTreeMap::<ObsKey, Observations>::new();

            for (k, values) in obs_rec.iter() {
                if let Some(converted) = solver.precise_epoch_correction(k.epoch, target) {
                    let mut key = k.clone();
                    key.epoch = converted;

                    new_rec.insert(key, values.clone());
                }
            }

            *obs_rec = new_rec;
        }
    }
}
