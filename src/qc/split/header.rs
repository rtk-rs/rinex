use crate::prelude::{qc::QcSplit, Duration, Epoch, Header};

impl QcSplit for Header {
    fn split_mut(&mut self, t: Epoch) -> Self {
        let mut rhs = self.clone();

        if let Some(obs) = &mut self.obs {
            if let Some(timeof) = &mut obs.timeof_first_obs {
                *timeof = std::cmp::min(*timeof, t);
            }
            if let Some(timeof) = &mut obs.timeof_last_obs {
                *timeof = std::cmp::min(*timeof, t);
            }
        }

        if let Some(obs) = &mut rhs.obs {
            if let Some(timeof) = &mut obs.timeof_first_obs {
                *timeof = std::cmp::min(*timeof, t);
            }
            if let Some(timeof) = &mut obs.timeof_last_obs {
                *timeof = std::cmp::max(*timeof, t);
            }
        }

        if let Some(doris) = &mut self.doris {
            if let Some(timeof) = &mut doris.timeof_first_obs {
                *timeof = std::cmp::min(*timeof, t);
            }
            if let Some(timeof) = &mut doris.timeof_last_obs {
                *timeof = std::cmp::min(*timeof, t);
            }
        }

        if let Some(doris) = &mut rhs.doris {
            if let Some(timeof) = &mut doris.timeof_first_obs {
                *timeof = std::cmp::min(*timeof, t);
            }
            if let Some(timeof) = &mut doris.timeof_last_obs {
                *timeof = std::cmp::max(*timeof, t);
            }
        }

        if let Some(ion) = &mut self.ionex {
            ion.epoch_of_first_map = std::cmp::min(ion.epoch_of_first_map, t);
            ion.epoch_of_last_map = std::cmp::min(ion.epoch_of_last_map, t);
        }

        if let Some(ion) = &mut rhs.ionex {
            ion.epoch_of_first_map = std::cmp::min(ion.epoch_of_first_map, t);
            ion.epoch_of_last_map = std::cmp::max(ion.epoch_of_last_map, t);
        }

        (a, b)
    }

    fn split_even_dt(&self, _: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        vec![self.clone()]
    }
}
