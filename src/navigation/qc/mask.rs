use qc_traits::{FilterItem, MaskFilter, MaskOperand};

use crate::{navigation::Record, prelude::Constellation};

fn mask_mut_geq(rec: &mut Record, target: &FilterItem) {
    match target {
        FilterItem::SvItem(items) => {
            rec.retain(|k, _| {
                let mut retained = true;
                for item in items.iter() {
                    if item.constellation == k.sv.constellation {
                        if k.sv.prn < item.prn {
                            retained = false;
                        }
                    }
                }
                retained
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_equal(rec: &mut Record, target: &FilterItem) {
    match target {
        FilterItem::SvItem(filter) => {
            rec.retain(|k, _| filter.contains(&k.sv));
        },
        FilterItem::ConstellationItem(filter) => {
            let mut broad_sbas_filter = false;
            for c in filter {
                broad_sbas_filter |= *c == Constellation::SBAS;
            }
            rec.retain(|k, _| {
                if broad_sbas_filter && k.sv.constellation.is_sbas() {
                    true
                } else {
                    filter.contains(&k.sv.constellation)
                }
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_ineq(rec: &mut Record, target: &FilterItem) {
    match target {
        FilterItem::SvItem(filter) => {
            rec.retain(|k, _| !filter.contains(&k.sv));
        },
        FilterItem::ConstellationItem(filter) => {
            let mut broad_sbas_filter = false;
            for c in filter {
                broad_sbas_filter |= *c == Constellation::SBAS;
            }
            rec.retain(|k, _| {
                if broad_sbas_filter && k.sv.constellation.is_sbas() {
                    false
                } else {
                    !filter.contains(&k.sv.constellation)
                }
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_leq(rec: &mut Record, target: &FilterItem) {
    match target {
        FilterItem::SvItem(items) => {
            rec.retain(|k, _| {
                let mut retained = true;
                for item in items.iter() {
                    if item.constellation == k.sv.constellation {
                        if k.sv.prn > item.prn {
                            retained = false;
                        }
                    }
                }
                retained
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_lt(rec: &mut Record, target: &FilterItem) {
    match target {
        FilterItem::SvItem(items) => {
            rec.retain(|k, _| {
                let mut retained = true;
                for item in items.iter() {
                    if item.constellation == k.sv.constellation {
                        if k.sv.prn >= item.prn {
                            retained = false;
                        }
                    }
                }
                retained
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_gt(rec: &mut Record, target: &FilterItem) {
    match target {
        FilterItem::SvItem(items) => {
            rec.retain(|k, _| {
                let mut retained = true;
                for item in items.iter() {
                    if item.constellation == k.sv.constellation {
                        if k.sv.prn <= item.prn {
                            retained = false;
                        }
                    }
                }
                retained
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

pub fn mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => mask_mut_equal(rec, &mask.item),
        MaskOperand::NotEquals => mask_mut_ineq(rec, &mask.item),
        MaskOperand::GreaterThan => mask_mut_gt(rec, &mask.item),
        MaskOperand::GreaterEquals => mask_mut_geq(rec, &mask.item),
        MaskOperand::LowerThan => mask_mut_lt(rec, &mask.item),
        MaskOperand::LowerEquals => mask_mut_leq(rec, &mask.item),
    }
}
