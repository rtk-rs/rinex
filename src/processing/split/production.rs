use crate::ProductionAttributes;

use qc_traits::Split;

impl Split for ProductionAttributes {
    fn split(&self, _t: hifitime::Epoch) -> (Self, Self)
    where
        Self: Sized,
    {
        let (mut a, mut b) = (self.clone(), self.clone());

        if let Some(details) = &mut a.v3_details {
            details.batch = 0;
        }

        if let Some(details) = &mut b.v3_details {
            details.batch = 1;
        }

        (a, b)
    }

    fn split_even_dt(&self, _dt: hifitime::Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        Default::default()
    }

    fn split_mut(&mut self, _t: hifitime::Epoch) -> Self {
        Default::default()
    }
}
