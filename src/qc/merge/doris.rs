use crate::{doris::Record, prelude::qc::MergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (k, v) in rhs.iter() {
        if let Some(lhs) = rec.get_mut(&k) {
            for (k, v) in v.signals.iter() {
                if let Some(lhs) = lhs.signals.get_mut(&k) {
                    if lhs.m1.is_none() && v.m1.is_some() {
                        lhs.m1 = Some(v.m1.unwrap());
                    }
                    if lhs.m2.is_none() && v.m2.is_some() {
                        lhs.m2 = Some(v.m2.unwrap());
                    }
                } else {
                    lhs.signals.insert(k.clone(), v.clone());
                }
            }
        } else {
            // new entry
            rec.insert(k.clone(), v.clone());
        }
    }
    Ok(())
}
