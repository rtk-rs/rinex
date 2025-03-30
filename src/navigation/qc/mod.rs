mod decim;
mod mask;

use qc_traits::{QcFilter, QcFilterType};

use crate::observation::{
    qc::{decim::decim_mut, mask::mask_mut},
    Record,
};

pub(crate) fn filter_mut(rec: &mut Record, f: &QcFilter) {
    match f.filter {
        QcFilterType::Mask(mask_op) => mask_mut(rec, mask_op, f.subset),
        QcFilterType::Decimation(decim_filter) => decim_mut(rec, &decim_filter, f.subset),
    }
}
