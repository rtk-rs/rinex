use crate::prelude::{qc::QcSplit, Epoch, ProductionAttributes};

impl QcSplit for ProductionAttributes {
    fn split_mut(&mut self, t: Epoch) -> Self {
        let mut rhs = self.clone();

        if let Some(details) = &mut self.v3_details {
            details.batch = 0;
        }

        if let Some(details) = &mut rhs.v3_details {
            details.batch = 1;
        }

        rhs
    }
}
