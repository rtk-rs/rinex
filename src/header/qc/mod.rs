use crate::header::Header;

mod decim;
mod mask;

use qc_traits::{html, Markup, QcFilter, QcFilterType, QcHtmlReporting, QcPreprocessing};

use decim::decim_mut;
use mask::mask_mut;

impl QcPreprocessing for Header {
    fn filter_mut(&mut self, f: &QcFilter) {
        match f.filter {
            QcFilterType::Decimation(decim) => decim_mut(decim, f.subset),
            QcFilterType::Mask(mask) => mask_mut(mask, f.subset),
        }
    }
}

#[cfg(feature = "qc")]
impl QcHtmlReporting for Header {
    fn render(&self) -> Markup {
        html! {
            tr {
                th { "Antenna" }
                @if let Some(antenna) = &self.rcvr_antenna {
                    td { (antenna.render()) }
                } @else {
                    td { "No information" }
                }
            }
            tr {
                th { "Receiver" }
                @ if let Some(rcvr) = &self.rcvr {
                    td { (rcvr.render()) }
                } else {
                    td { "No information" }
                }
            }
        }
    }
}
