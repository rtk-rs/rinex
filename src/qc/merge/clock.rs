use crate::{clock::Record, prelude::qc::MergeError};

use super::merge_mut_option;

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (rhs_epoch, rhs_content) in rhs.iter() {
        if let Some(lhs_content) = rec.get_mut(rhs_epoch) {
            for (rhs_key, rhs_prof) in rhs_content.iter() {
                if let Some(lhs_prof) = lhs_content.get_mut(rhs_key) {
                    // enhance only, if possible
                    merge_mut_option(&mut lhs_prof.drift, &rhs_prof.drift);
                    merge_mut_option(&mut lhs_prof.drift_dev, &rhs_prof.drift_dev);
                    merge_mut_option(&mut lhs_prof.drift_change, &rhs_prof.drift_change);
                    merge_mut_option(&mut lhs_prof.drift_change_dev, &rhs_prof.drift_change_dev);
                } else {
                    lhs_content.insert(rhs_key.clone(), rhs_prof.clone());
                }
            }
        } else {
            rec.insert(*rhs_epoch, rhs_content.clone());
        }
    }
    Ok(())
}
