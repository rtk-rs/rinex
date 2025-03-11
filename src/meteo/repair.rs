//! Meteo RINEX repair implementation
use crate::meteo::Record;
use qc_traits::Repair;

pub fn repair_mut(_rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => {},
    }
}
