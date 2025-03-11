use crate::header::Header;
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

fn header_mask_eq(hd: &mut Header, item: &FilterItem) {
    match item {
        FilterItem::ConstellationItem(constellations) => {
            hd.dcb_compensations
                .retain(|dcb| constellations.contains(&dcb.constellation));
            hd.pcv_compensations
                .retain(|pcv| constellations.contains(&pcv.constellation));
        },
        _ => {},
    }
}

fn header_mask_neq(hd: &mut Header, item: &FilterItem) {
    match item {
        FilterItem::ConstellationItem(constellations) => {
            hd.dcb_compensations
                .retain(|dcb| !constellations.contains(&dcb.constellation));
            hd.pcv_compensations
                .retain(|pcv| !constellations.contains(&pcv.constellation));
        },
        _ => {},
    }
}

pub(crate) fn header_mask_mut(hd: &mut Header, f: &MaskFilter) {
    match f.operand {
        MaskOperand::Equals => header_mask_eq(hd, &f.item),
        MaskOperand::NotEquals => header_mask_neq(hd, &f.item),
        MaskOperand::GreaterThan => {},
        MaskOperand::GreaterEquals => {},
        MaskOperand::LowerThan => {},
        MaskOperand::LowerEquals => {},
    }
    if let Some(obs) = &mut hd.obs {
        obs.mask_mut(f);
    }
    if let Some(met) = &mut hd.meteo {
        met.mask_mut(f);
    }
    if let Some(ionex) = &mut hd.ionex {
        ionex.mask_mut(f);
    }
    if let Some(doris) = &mut hd.doris {
        doris.mask_mut(f);
    }
}
