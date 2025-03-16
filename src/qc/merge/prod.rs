use crate::{
    prelude::qc::{QcMerge, QcMergeError},
    prod::{DataSource, ProductionAttributes},
};

use super::merge_mut_option;

impl QcMerge for ProductionAttributes {
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), QcMergeError> {
        merge_mut_option(&mut self.region, &rhs.region);
        merge_mut_option(&mut self.v3_details, &rhs.v3_details);
        if let Some(lhs) = &mut self.v3_details {
            if let Some(rhs) = &rhs.v3_details {
                merge_mut_option(&mut lhs.ffu, &rhs.ffu);
                /*
                 * Data source is downgraded to "Unknown"
                 * in case we wind up cross mixing data sources
                 */
                if lhs.data_src != rhs.data_src {
                    lhs.data_src = DataSource::Unknown;
                }
            }
        }
        Ok(())
    }
}
