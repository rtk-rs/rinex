use crate::doris::Record;

/// Repairs all Zero (=null) values in [Record]
pub fn zero_repair_mut(rec: &mut Record) {
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
