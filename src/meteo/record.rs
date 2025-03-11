use crate::{epoch, prelude::*, types::Type, version, Observable};

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "processing")]
use qc_traits::{DecimationFilter, DecimationFilterType, FilterItem, MaskFilter, MaskOperand};

/*
 * Meteo RINEX specific record type.
 */
pub type Record = BTreeMap<Epoch, HashMap<Observable, f64>>;

/*
 * Returns true if given line matches a new Meteo Record Epoch.
 * We use this when browsing a RINEX file, to determine whether
 * we should initiate the parsing of a meteo record entry.
 */
pub(crate) fn is_new_epoch(line: &str, v: version::Version) -> bool {
    if v.major < 3 {
        let min_len = " 15  1  1  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse_utc(datestr).is_ok() // valid epoch descriptor
    } else {
        let min_len = " 2021  1  7  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse_utc(datestr).is_ok() // valid epoch descriptor
    }
}

#[derive(Error, Debug)]
/// Meteo Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse epoch")]
    EpochParsingError(#[from] epoch::ParsingError),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/*
 * Meteo record entry parsing method
 */
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<(Epoch, HashMap<Observable, f64>), Error> {
    let mut lines = content.lines();
    let mut line = lines.next().unwrap();

    let mut map: HashMap<Observable, f64> = HashMap::with_capacity(3);

    let mut offset: usize = 18; // YY
    if header.version.major > 2 {
        offset += 2; // YYYY
    }

    let epoch = epoch::parse_utc(&line[0..offset])?;

    let codes = &header.meteo.as_ref().unwrap().codes;
    let nb_codes = codes.len();
    let nb_lines: usize = num_integer::div_ceil(nb_codes, 8);
    let mut code_index: usize = 0;

    for i in 0..nb_lines {
        for _ in 0..8 {
            let code = &codes[code_index];
            let obs: Option<f64> = match f64::from_str(line[offset..offset + 7].trim()) {
                Ok(f) => Some(f),
                Err(_) => None,
            };

            if let Some(obs) = obs {
                map.insert(code.clone(), obs);
            }
            code_index += 1;
            if code_index >= nb_codes {
                break;
            }

            offset += 7;
            if offset >= line.len() {
                break;
            }
        } // 1:8

        if i < nb_lines - 1 {
            if let Some(l) = lines.next() {
                line = l;
            } else {
                break;
            }
        }
    } // nb lines
    Ok((epoch, map))
}

/*
 * Epoch formatter
 * is used when we're dumping a Meteo RINEX record entry
 */
pub(crate) fn fmt_epoch(
    epoch: &Epoch,
    data: &HashMap<Observable, f64>,
    header: &Header,
) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    lines.push_str(&format!(
        " {}",
        epoch::format(*epoch, Type::MeteoData, header.version.major)
    ));
    let observables = &header.meteo.as_ref().unwrap().codes;
    let mut index = 0;
    for obscode in observables {
        index += 1;
        if let Some(data) = data.get(obscode) {
            lines.push_str(&format!("{:7.1}", data));
        } else {
            lines.push_str("       ");
        }
        if (index % 8) == 0 {
            lines.push('\n');
        }
    }
    lines.push('\n');
    Ok(lines)
}

#[cfg(feature = "qc")]
use qc_traits::MergeError;

#[cfg(feature = "qc")]
pub(crate) fn merge_mut(lhs: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (epoch, observations) in rhs.iter() {
        if let Some(oobservations) = lhs.get_mut(epoch) {
            for (observation, data) in observations.iter() {
                if !oobservations.contains_key(observation) {
                    // new observation
                    oobservations.insert(observation.clone(), *data);
                }
            }
        } else {
            // new epoch
            lhs.insert(*epoch, observations.clone());
        }
    }
    Ok(())
}

#[cfg(feature = "processing")]
pub(crate) fn meteo_mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e == *epoch),
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
                    rec.retain(|_, data| {
                        data.retain(|code, _| observables.contains(code));
                        !data.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e != *epoch),
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
                    rec.retain(|_, data| {
                        data.retain(|code, _| !observables.contains(code));
                        !data.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::GreaterEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e >= *epoch),
            _ => {},
        },
        MaskOperand::GreaterThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e > *epoch),
            _ => {},
        },
        MaskOperand::LowerEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e <= *epoch),
            _ => {},
        },
        MaskOperand::LowerThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e < *epoch),
            _ => {},
        },
    }
}

#[cfg(feature = "processing")]
pub(crate) fn meteo_decim_mut(rec: &mut Record, f: &DecimationFilter) {
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
            rec.retain(|e, _| {
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
    use super::*;
    #[test]
    fn test_new_epoch() {
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4  9 55  0  997.9   -6.4   54.2    2.9  342.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4 10  0  0  997.9   -6.3   55.4    3.4  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 08  1  1  0  0  1 1018.0   25.1   75.9    1.4   95.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 2021  1  7  0  0  0  993.3   23.0   90.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 4, minor: 0 }
        ));
    }
}
