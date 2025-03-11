use crate::doris::Record;

use qc_traits::Repair;

/// Repairs all Zero (=null) values in [Record]
fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, obs| {
        obs.signals.retain(|key, signal| {
            if key.observable.is_pseudo_range_observable()
                || key.observable.is_phase_range_observable()
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
