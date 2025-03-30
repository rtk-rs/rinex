use crate::prelude::{Epoch, ProductionAttributes};

use qc_traits::QcSplit;

impl QcSplit for ProductionAttributes {
    fn split_mut(&mut self, t: Epoch) -> Self {
        let mut copy = self.clone();

        if let Some(details) = &mut self.v3_details {
            details.batch = 0;
            copy.v3_details.unwrap().batch = 1;
        }

        copy
    }

    fn split_even_dt(&self, dt: hifitime::Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        Default::default()
    }
}
