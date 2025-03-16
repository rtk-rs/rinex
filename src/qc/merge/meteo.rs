use crate::{meteo::Record, prelude::qc::QcMergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), QcMergeError> {
    for (k, v) in rhs.iter() {
        if rec.get(&k).is_none() {
            rec.insert(k.clone(), *v);
        }
    }
    Ok(())
}
