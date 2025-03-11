//! "csv" description helps the testbench describe complex stuff

use itertools::Itertools;
use std::str::FromStr;

use crate::prelude::{Constellation, Observable, SV};

/// CSV to [Observable]s
pub fn observables_csv(observable_csv: &str) -> Vec<Observable> {
    observable_csv
        .split(',')
        .map(|c| Observable::from_str(c.trim()).unwrap())
        .collect::<Vec<Observable>>()
        .into_iter()
        .unique()
        .collect()
}

/// CSV to [Constellation]s
pub fn gnss_csv(gnss_csv: &str) -> Vec<Constellation> {
    gnss_csv
        .split(',')
        .map(|c| Constellation::from_str(c.trim()).unwrap())
        .collect::<Vec<Constellation>>()
        .into_iter()
        .unique()
        .sorted()
        .collect()
}

/// CSV to [SV]s
pub fn sv_csv(gnss_csv: &str) -> Vec<SV> {
    gnss_csv
        .split(',')
        .map(|c| {
            let trimmed = c.trim();
            SV::from_str(trimmed)
                .unwrap_or_else(|_| panic!("invalid test SV specs: \"{}\"", trimmed))
        })
        .collect::<Vec<SV>>()
        .into_iter()
        .unique()
        .sorted()
        .collect()
}
