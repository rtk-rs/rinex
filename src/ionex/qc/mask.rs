use crate::ionex::Record;
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

pub fn mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch == epoch),
            _ => {}, // does not apply
        },
        MaskOperand::NotEquals => match mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch != epoch),
            _ => {}, // does not apply
        },
        MaskOperand::GreaterEquals => match mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch >= epoch),
            _ => {}, // does not apply
        },
        MaskOperand::GreaterThan => match mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch > epoch),
            _ => {}, // does not apply
        },
        MaskOperand::LowerEquals => match mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch <= epoch),
            _ => {}, // does not apply
        },
        MaskOperand::LowerThan => match mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch < epoch),
            _ => {}, // does not apply
        },
    }
}
