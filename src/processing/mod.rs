use qc_traits::Preprocessing;

use crate::prelude::Rinex;

mod decim;
mod repair;
mod split;

impl Preprocessing for Rinex {}
