//! Observation RINEX repair implementation
use crate::observation::Record;

/// Repairs all Zero (=null) values in [Record]
pub fn zero_repair_mut(rec: &mut Record) {
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
