use crate::{
    ionex::Record,
    prelude::qc::{QcMerge, QcMergeError},
};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), QcMergeError> {
    for (k, v) in rhs.iter() {
        if let Some(tec) = rec.get_mut(&k) {
            tec.merge_mut(&v)?;
        } else {
            // new TEC value in space and/or time
            rec.insert(*k, v.clone());
        }
    }
    Ok(())
}
