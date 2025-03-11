// use crate::observation::Record as ObsRecord;
use std::str::FromStr;

use crate::prelude::{Observable, Rinex, Station};

/*
 * Verifies observable list
 */
pub fn check_observables(rinex: &Rinex, observables: &[&str]) {
    let expected = observables
        .iter()
        .map(|desc| Observable::from_str(desc).unwrap())
        .collect::<Vec<_>>();
    let header = rinex.header.doris.as_ref().expect("missing header fields!");
    assert_eq!(header.observables, expected);
}

/*
 * Verifies Station list
 */
pub fn check_stations(rinex: &Rinex, stations: &[&str]) {
    let mut expected = stations
        .iter()
        .map(|s| Station::from_str(s).unwrap())
        .collect::<Vec<_>>();
    let mut header = rinex
        .header
        .doris
        .as_ref()
        .expect("missing header fields!")
        .clone();
    assert_eq!(header.stations.sort(), expected.sort());
}
