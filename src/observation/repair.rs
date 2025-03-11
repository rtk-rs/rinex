//! Observation RINEX repair implementation
use crate::observation::Record;

use qc_traits::Repair;

/// Repairs all Zero (=null) values in [Record]
fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, obs| {
        obs.signals.retain(|signal| {
            if signal.observable.is_pseudo_range_observable()
                || signal.observable.is_phase_range_observable()
            {
                signal.value > 0.0
            } else {
                true
            }
        });
        !obs.signals.is_empty()
    });
}

/// Applies [Repair] to [Record]
pub fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
    }
}
