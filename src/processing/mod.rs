use qc_traits::Preprocessing;

use crate::prelude::Rinex;

mod decim;
mod repair;
mod split;
mod timeshift;

impl Preprocessing for Rinex {}
