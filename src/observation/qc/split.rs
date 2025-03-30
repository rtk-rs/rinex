use qc_traits::QcSplit;

use crate::observation::HeaderFields;

impl QcSplit for HeaderFields {
    fn split_mut(&mut self, t: hifitime::Epoch) -> Self {
        let mut copy = self.clone();

        if let Some(timeof) = &mut self.timeof_first_obs {
            *timeof = std::cmp::min(*timeof, t);
            copy.timeof_first_obs = Some(std::cmp::max(*timeof, t));
        }

        if let Some(timeof) = &mut self.timeof_last_obs {
            *timeof = std::cmp::min(*timeof, t);
            copy.timeof_last_obs = Some(std::cmp::min(*timeof, t));
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
