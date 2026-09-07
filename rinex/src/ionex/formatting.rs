//! IONEX maps formatting

use crate::{
    epoch::format_ionex_utc,
    fmt_rinex,
    ionex::{IonexKey, Quantized, QuantizedCoordinates, Record, TEC},
    linspace::Linspace,
    prelude::{Epoch, Header},
    FormattingError,
};

use itertools::Itertools;

use std::io::{BufWriter, Write};

/// Values per line of a map, I5 each
const VALUES_PER_LINE: usize = 16;

/// Value of a grid point without estimate
const NON_AVAILABLE: i64 = 9999;

/// Points of a [Linspace], from start to end (included).
/// A null spacing describes a single point.
fn points(space: &Linspace) -> impl Iterator<Item = f64> + '_ {
    let count = if space.spacing == 0.0 {
        1
    } else {
        ((space.end - space.start) / space.spacing).round() as usize + 1
    };

    (0..count).map(move |nth| space.start + nth as f64 * space.spacing)
}

/// Formats [Record] following the IONEX specifications:
/// the TEC maps in chronological order, numbered from 1, then
/// the RMS maps when the record carries RMS estimates.
/// Height maps are not supported.
pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    let specs = header
        .ionex
        .as_ref()
        .ok_or(FormattingError::NoGridDefinition)?;

    let epochs = record
        .keys()
        .map(|k| k.epoch)
        .unique()
        .sorted()
        .collect::<Vec<_>>();

    let exponent = specs.exponent;

    for (nth, t) in epochs.iter().enumerate() {
        format_map(w, "TEC", nth + 1, *t, record, specs, |tec| {
            Some(tec.quantized_tecu(exponent))
        })?;
    }

    if record.values().any(|tec| tec.rms_tec().is_some()) {
        for (nth, t) in epochs.iter().enumerate() {
            format_map(w, "RMS", nth + 1, *t, record, specs, |tec| {
                tec.quantized_rms(exponent)
            })?;
        }
    }

    writeln!(w, "{}", fmt_rinex("", "END OF FILE"))?;

    Ok(())
}

/// Formats one map of `kind` ("TEC" or "RMS") for [Epoch] `t`,
/// browsing the reference grid of the header: for each altitude,
/// for each latitude, the values along the longitudes.
fn format_map<W: Write, F: Fn(&TEC) -> Option<i64>>(
    w: &mut BufWriter<W>,
    kind: &str,
    nth: usize,
    t: Epoch,
    record: &Record,
    specs: &crate::ionex::HeaderFields,
    value_of: F,
) -> Result<(), FormattingError> {
    let grid = &specs.grid;

    // coordinates are quantized like the parser does
    let lat_exponent = Quantized::find_exponent(grid.latitude.spacing);
    let long_exponent = Quantized::find_exponent(grid.longitude.spacing);
    let alt_exponent = Quantized::find_exponent(grid.height.spacing);

    writeln!(
        w,
        "{}",
        fmt_rinex(&format!("{:6}", nth), &format!("START OF {} MAP", kind))
    )?;

    writeln!(
        w,
        "{}",
        fmt_rinex(&format_ionex_utc(t), "EPOCH OF CURRENT MAP")
    )?;

    for altitude_km in points(&grid.height) {
        for latitude_ddeg in points(&grid.latitude) {
            writeln!(
                w,
                "{}",
                fmt_rinex(
                    &format!(
                        "  {:6.1}{:6.1}{:6.1}{:6.1}{:6.1}",
                        latitude_ddeg,
                        grid.longitude.start,
                        grid.longitude.end,
                        grid.longitude.spacing,
                        altitude_km,
                    ),
                    "LAT/LON1/LON2/DLON/H"
                )
            )?;

            let mut written = 0;

            for longitude_ddeg in points(&grid.longitude) {
                let coordinates = QuantizedCoordinates::new(
                    latitude_ddeg,
                    lat_exponent,
                    longitude_ddeg,
                    long_exponent,
                    altitude_km,
                    alt_exponent,
                );

                let key = IonexKey {
                    epoch: t,
                    coordinates,
                };

                let value = record
                    .get(&key)
                    .and_then(&value_of)
                    .unwrap_or(NON_AVAILABLE);

                write!(w, "{:5}", value)?;

                written += 1;

                if written % VALUES_PER_LINE == 0 {
                    writeln!(w)?;
                }
            }

            if written % VALUES_PER_LINE != 0 {
                writeln!(w)?;
            }
        }
    }

    writeln!(
        w,
        "{}",
        fmt_rinex(&format!("{:6}", nth), &format!("END OF {} MAP", kind))
    )?;

    Ok(())
}
