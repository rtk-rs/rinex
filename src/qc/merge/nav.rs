use crate::{navigation::Record, prelude::qc::MergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (k, v) in rhs.iter() {
        if rec.get(&k).is_none() {
            rec.insert(k.clone(), v.clone());
        }
    }
    Ok(())
}
