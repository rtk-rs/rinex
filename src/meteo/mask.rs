use crate::{meteo::Record, prelude::Observable};
use std::str::FromStr;

use qc_traits::{FilterItem, MaskFilter, MaskOperand};

/// Applies [MaskFilter] to [Record]
pub fn mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch == *epoch),
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if observables.len() > 0 {
                    rec.retain(|k, _| observables.contains(&k.observable));
                }
            },
            _ => {},
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch != *epoch),
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if observables.len() > 0 {
                    rec.retain(|k, _| !observables.contains(&k.observable));
                }
            },
            _ => {},
        },
        MaskOperand::GreaterEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch >= *epoch),
            _ => {},
        },
        MaskOperand::GreaterThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch > *epoch),
            _ => {},
        },
        MaskOperand::LowerEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch <= *epoch),
            _ => {},
        },
        MaskOperand::LowerThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch < *epoch),
            _ => {},
        },
    }
}
