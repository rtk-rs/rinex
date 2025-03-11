use crate::navigation::Record;
use qc_traits::Repair;

// Zero fields are invalid Ephemeris content (according to ICD specs).
// Depending on quality of publisher, it may be required to fix
// the RINex prior post processing.
fn repair_zero_mut(_rec: &mut Record) {}

pub fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
    }
}
