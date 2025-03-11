use crate::doris::Record;
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

pub fn mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch == *epoch),
            FilterItem::ComplexItem(_filter) => {
                //rec.retain(|_, stations| {
                //    stations.retain(|_, obs| {
                //        obs.retain(|code, _| filter.contains(code));
                //        !obs.is_empty()
                //    });
                //    !stations.is_empty()
                //});
            },
            _ => {}, //TODO: some other types could apply, like SNR..
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch != *epoch),
            FilterItem::ComplexItem(_filter) => {
                //rec.retain(|_, stations| {
                //    stations.retain(|_, obs| {
                //        obs.retain(|code, _| !filter.contains(code));
                //        !obs.is_empty()
                //    });
                //    !stations.is_empty()
                //});
            },
            _ => {}, //TODO: some other types could apply, like SNR..
        },
        _ => {},
    }
}
