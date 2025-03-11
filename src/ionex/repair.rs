use crate::ionex::Record;

use qc_traits::Repair;

/// Applies [Repair] to [Record]
pub fn repair_mut(_rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => {},
    }
}
