use crate::doris::Record;

use qc_traits::{QcMaskOperand, QcSubset};

pub fn mask_mut(rec: &mut Record, operand: QcMaskOperand, subset: &QcSubset) {
    match operand {
        QcMaskOperand::Equals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch == *epoch),
            _ => {},
        },
        QcMaskOperand::NotEquals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch != *epoch),
            _ => {},
        },
        QcMaskOperand::GreaterEquals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch >= *epoch),
            _ => {},
        },
        QcMaskOperand::GreaterThan => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch > *epoch),
            _ => {},
        },
        QcMaskOperand::LowerEquals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch <= *epoch),
            _ => {},
        },
        QcMaskOperand::LowerThan => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch < *epoch),
            _ => {},
        },
    }
}
