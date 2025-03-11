use hifitime::Epoch;
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

use crate::{
    doris::Station,
    epoch::{parse_in_timescale, ParsingError as EpochParsingError},
    header::Header,
    observable::Observable,
    observation::EpochFlag,
    prelude::TimeScale,
};

#[cfg(feature = "processing")]
use qc_traits::{DecimationFilter, DecimationFilterType, FilterItem, MaskFilter, MaskOperand};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ObservationData {
    /// Actual measurement
    pub value: f64,
    /// Flag m1
    pub m1: Option<u8>,
    /// Flag m2
    pub m2: Option<u8>,
}

/// DORIS RINEX Record content.
/// Measurements are stored by Kind, by Station and by TAI sampling instant.
pub type Record =
    BTreeMap<(Epoch, EpochFlag), BTreeMap<Station, HashMap<Observable, ObservationData>>>;

/// Returns true if following line matches a new DORIS measurement
pub(crate) fn is_new_epoch(line: &str) -> bool {
    line.starts_with('>')
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] EpochParsingError),
    #[error("failed to parse data")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/// DORIS measurement parsing process
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<
    (
        (Epoch, EpochFlag),
        BTreeMap<Station, HashMap<Observable, ObservationData>>,
    ),
    Error,
> {
    let mut obs_idx = 0usize;
    let mut epoch = Epoch::default();
    let flag = EpochFlag::default();
    let mut station = Option::<Station>::None;
    let mut buffer = BTreeMap::<Station, HashMap<Observable, ObservationData>>::new();

    let doris = header
        .doris
        .as_ref()
        .expect("missing header field(s): badly formed DORIS RINEX");

    let observables = &doris.observables;
    let stations = &doris.stations;

    assert!(
        !stations.is_empty(),
        "badly formed DORIS RINEX: no stations defined"
    );
    assert!(
        !observables.is_empty(),
        "badly formed DORIS RINEX: no observables defined"
    );

    for (lindex, line) in content.lines().enumerate() {
        match lindex {
            0 => {
                /* 1st line gives TAI timestamp, flag, clock offset */
                let line = line.split_at(2).1; // "> "
                let offset = "YYYY MM DD HH MM SS.NNNNNNNNN  0".len();
                let (date, _rem) = line.split_at(offset);
                epoch = parse_in_timescale(date, TimeScale::TAI)?;
            },
            _ => {
                let (id, _remainder) = line.split_at(4);
                //println!("ID : \"{}\" - REMAINDER : \"{}\"", id, remainder); //DBEUG

                if obs_idx == 0 {
                    // parse station identifier
                    assert!(id.len() > 1, "badly formed DORIS station identifier");
                    let key = id[1..]
                        .trim()
                        .parse::<u16>()
                        .unwrap_or_else(|e| panic!("failed to identify DORIS station: {:?}", e));

                    station = stations
                        .iter()
                        .filter(|station| station.key == key)
                        .reduce(|k, _| k)
                        .cloned();
                }

                let identified_station =
                    station.as_ref().expect("failed to identify DORIS station");

                // consume this line
                let mut offset = 5;
                let max_offset = line.len();
                while offset < line.len() {
                    let content = &line[offset..std::cmp::min(max_offset, offset + 16)];
                    let obs = &content[..12];
                    let m1 = &content[12..13].trim();
                    let m2 = &content[13..14].trim();

                    //println!("obs \"{}\"", obs); //DEBUG
                    //println!("m1 \"{}\"", m1); //DEBUG
                    //println!("m2 \"{}\"", m2); //DEBUG

                    let value = obs
                        .trim()
                        .parse::<f64>()
                        .unwrap_or_else(|e| panic!("failed to parse observation: {:?}", e));

                    let m1 = if !m1.is_empty() {
                        Some(m1.parse::<u8>().unwrap_or_else(|e| {
                            panic!("failed to parse observation m1 flag: {:?}", e)
                        }))
                    } else {
                        None
                    };

                    let m2 = if !m2.is_empty() {
                        Some(m2.parse::<u8>().unwrap_or_else(|e| {
                            panic!("failed to parse observation m2 flag: {:?}", e)
                        }))
                    } else {
                        None
                    };

                    let observable = observables.get(obs_idx).unwrap_or_else(|| {
                        panic!(
                            "failed to determine observable for {:?}({:?}) @ {}",
                            identified_station, epoch, obs_idx
                        )
                    });

                    let obsdata = ObservationData { value, m1, m2 };

                    if let Some(station) = buffer.get_mut(identified_station) {
                        station.insert(observable.clone(), obsdata);
                    } else {
                        let inner =
                            HashMap::from_iter([(Observable::default(), obsdata)].into_iter());
                        buffer.insert(identified_station.clone(), inner);
                    }

                    offset += 16;
                    obs_idx += 1;
                }
                if obs_idx == observables.len() {
                    obs_idx = 0;
                    station = None;
                }
            },
        }
    }
    Ok(((epoch, flag), buffer))
}

#[cfg(feature = "processing")]
pub(crate) fn doris_mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e == *epoch),
            FilterItem::ComplexItem(_filter) => {
                //rec.retain(|_, stations| {
                //    stations.retain(|_, obs| {
                //        obs.retain(|code, _| filter.contains(code));
                //        !obs.is_empty()
                //    });
                //    !stations.is_empty()
                //});
            },
            _ => {}, //TODO: some other types could apply, like SNR..
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e != *epoch),
            FilterItem::ComplexItem(_filter) => {
                //rec.retain(|_, stations| {
                //    stations.retain(|_, obs| {
                //        obs.retain(|code, _| !filter.contains(code));
                //        !obs.is_empty()
                //    });
                //    !stations.is_empty()
                //});
            },
            _ => {}, //TODO: some other types could apply, like SNR..
        },
        _ => {},
    }
}

#[cfg(feature = "processing")]
pub(crate) fn doris_decim_mut(rec: &mut Record, f: &DecimationFilter) {
    if f.item.is_some() {
        todo!("targetted decimation not supported yet");
    }
    match f.filter {
        DecimationFilterType::Modulo(r) => {
            let mut i = 0;
            rec.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
        },
        DecimationFilterType::Duration(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            rec.retain(|(e, _), _| {
                if let Some(last) = last_retained {
                    let dt = *e - last;
                    if dt >= interval {
                        last_retained = Some(*e);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*e);
                    true // always retain 1st epoch
                }
            });
        },
    }
}

#[cfg(test)]
mod test {
    use super::{is_new_epoch, parse_epoch};
    use crate::{
        doris::record::ObservationData,
        doris::HeaderFields as DorisHeader,
        doris::Station,
        prelude::{DOMESTrackingPoint, DOMES},
        Epoch, EpochFlag, Header, Observable,
    };
    use std::str::FromStr;
    #[test]
    fn new_epoch() {
        for (desc, expected) in [
            (
                "> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "> 2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "  2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            (
                "  2022 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            ("test", false),
        ] {
            assert_eq!(is_new_epoch(desc), expected);
        }
    }
    #[test]
    fn valid_epoch() {
        let mut header = Header::default();
        let mut doris = DorisHeader::default();
        for obs in ["L1", "L2", "C1", "C2", "W1", "W2", "F", "P", "T", "H"] {
            let obs = Observable::from_str(obs).unwrap();
            doris.observables.push(obs);
        }
        for station in [
            "D01  THUB THULE                         43001S005  3   0",
            "D02  SVBC NY-ALESUND II                 10338S004  4   0",
        ] {
            let station = Station::from_str(station).unwrap();
            doris.stations.push(station);
        }
        header.doris = Some(doris);

        let content = "> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0 
D01  -3237877.052    -2291024.044    21903595.62311  21903633.08011      -113.100 7
          -98.400 7       437.801        1002.000 1       -20.000 1        82.000 1
D02  -2069899.788     -407871.014     4677242.25714   4677392.20614      -119.050 7
         -111.000 7       437.801        1007.000 0        -2.000 0        74.000 0";

        let ((e, flag), content) =
            parse_epoch(&header, content).expect("failed to parse DORIS epoch");

        assert_eq!(
            e,
            Epoch::from_str("2024-01-01T00:00:28.999947700 TAI").unwrap(),
            "parsed wrong epoch"
        );
        assert_eq!(flag, EpochFlag::Ok, "parsed wrong epoch flag");

        let station = Station {
            key: 1,
            gen: 3,
            k_factor: 0,
            label: "THUB".to_string(),
            site: "THULE".to_string(),
            domes: DOMES {
                site: 1,
                area: 430,
                sequential: 5,
                point: DOMESTrackingPoint::Instrument,
            },
        };
        let values = content
            .get(&station)
            .unwrap_or_else(|| panic!("failed to identify {:?}", station));

        for (observable, data) in [
            (
                Observable::from_str("L1C").unwrap(),
                ObservationData {
                    m1: None,
                    m2: None,
                    value: -3237877.052,
                },
            ),
            (
                Observable::from_str("L2").unwrap(),
                ObservationData {
                    m1: None,
                    m2: None,
                    value: -2291024.044,
                },
            ),
            (
                Observable::from_str("C1").unwrap(),
                ObservationData {
                    m1: Some(1),
                    m2: Some(1),
                    value: 21903595.623,
                },
            ),
            (
                Observable::from_str("C2").unwrap(),
                ObservationData {
                    m1: Some(1),
                    m2: Some(1),
                    value: 21903633.080,
                },
            ),
            (
                Observable::from_str("W1").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(7),
                    value: -113.100,
                },
            ),
            (
                Observable::from_str("W2").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(7),
                    value: -98.400,
                },
            ),
            (
                Observable::from_str("F").unwrap(),
                ObservationData {
                    m1: None,
                    m2: None,
                    value: 437.801,
                },
            ),
            (
                Observable::from_str("P").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(1),
                    value: 1002.000,
                },
            ),
            (
                Observable::from_str("T").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(1),
                    value: -20.0,
                },
            ),
            (
                Observable::from_str("H").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(1),
                    value: 82.0,
                },
            ),
        ] {
            let value = values
                .get(&observable)
                .unwrap_or_else(|| panic!("failed to identify {:?}", observable));
            assert_eq!(value, &data, "wrong value parsed for {:?}", observable);
        }

        let station = Station {
            key: 2,
            gen: 4,
            k_factor: 0,
            label: "SVBC".to_string(),
            site: "NY-ALESUND II".to_string(),
            domes: DOMES {
                site: 38,
                area: 103,
                sequential: 4,
                point: DOMESTrackingPoint::Instrument,
            },
        };
        let values = content
            .get(&station)
            .unwrap_or_else(|| panic!("failed to identify {:?}", station));

        for (observable, data) in [
            (
                Observable::from_str("L1C").unwrap(),
                ObservationData {
                    m1: None,
                    m2: None,
                    value: -2069899.788,
                },
            ),
            (
                Observable::from_str("L2").unwrap(),
                ObservationData {
                    m1: None,
                    m2: None,
                    value: -407871.014,
                },
            ),
            (
                Observable::from_str("C1").unwrap(),
                ObservationData {
                    m1: Some(1),
                    m2: Some(4),
                    value: 4677242.257,
                },
            ),
            (
                Observable::from_str("C2").unwrap(),
                ObservationData {
                    m1: Some(1),
                    m2: Some(4),
                    value: 4677392.206,
                },
            ),
            (
                Observable::from_str("W1").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(7),
                    value: -119.050,
                },
            ),
            (
                Observable::from_str("W2").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(7),
                    value: -111.000,
                },
            ),
            (
                Observable::from_str("F").unwrap(),
                ObservationData {
                    m1: None,
                    m2: None,
                    value: 437.801,
                },
            ),
            (
                Observable::from_str("P").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(0),
                    value: 1007.000,
                },
            ),
            (
                Observable::from_str("T").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(0),
                    value: -2.000,
                },
            ),
            (
                Observable::from_str("H").unwrap(),
                ObservationData {
                    m1: None,
                    m2: Some(0),
                    value: 74.0,
                },
            ),
        ] {
            let value = values
                .get(&observable)
                .unwrap_or_else(|| panic!("failed to identify {:?}", observable));
            assert_eq!(value, &data, "wrong value parsed for {:?}", observable);
        }
    }
}
