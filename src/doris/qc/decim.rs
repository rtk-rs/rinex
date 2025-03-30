use crate::{doris::Record, prelude::Epoch};

use qc_traits::{QcDecimationFilter, QcSubset};

pub fn decim_mut(rec: &mut Record, f: &QcDecimationFilter, subset: &QcSubset) {
    if subset != QcSubset::All {
        unimplemented!("scoped decimation: not supported yet!");
    }

    match f {
        QcDecimationFilter::Duration(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            rec.retain(|k, _| {
                if let Some(last) = last_retained {
                    let dt = k.epoch - last;
                    if dt >= *interval {
                        last_retained = Some(k.epoch);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(k.epoch);
                    true // always retain 1st epoch
                }
            });
        },
        QcDecimationFilter::Modulo(modulo) => {
            let mut i = 0;
            rec.retain(|_, _| {
                let retained = (i % modulo) == 0;
                i += 1;
                retained
            });
        },
    }
}
