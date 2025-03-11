use crate::{
    fmt_rinex,
    ionex::{IonexKey, QuantizedCoordinates, Record},
    prelude::Header,
    FormattingError,
};

use itertools::Itertools;

use std::io::{BufWriter, Write};

pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    const NUM_LONGITUDES_PER_LINE: usize = 16;

    let specs = header
        .ionex
        .as_ref()
        .ok_or(FormattingError::NoGridDefinition)?;

    // browse grid:
    // - for each altitude (km)
    // - - browse latitude (starting on northernmost.. to southernmost)
    // - - - browse longitude (starting on easternmost.. to westernmost)
    let grid = &specs.grid;

    let (altitude_low_km, altitude_high_km, altitude_spacing_km) =
        (grid.height.start, grid.height.end, grid.height.spacing);

    let (latitude_north_ddeg, latitude_south_ddeg, latitude_spacing_ddeg) = (
        grid.latitude.start,
        grid.latitude.end,
        grid.latitude.spacing,
    );

    let (longitude_east_ddeg, longitude_west_ddeg, longitude_spacing_ddeg) = (
        grid.longitude.start,
        grid.longitude.end,
        grid.longitude.spacing,
    );

    let mut nth_map = 0;
    let mut has_h = false;
    let mut has_rms = false;

    for t in record.keys().map(|k| k.epoch).unique().sorted() {
        // format all TEC maps
        writeln!(
            w,
            "{}",
            fmt_rinex(&format!("{:6}", nth_map), "START OF TEC MAP")
        )?;

        let mut altitude_km = altitude_low_km;

        while altitude_km < altitude_high_km {
            let mut latitude_ddeg = latitude_north_ddeg;
            while latitude_ddeg < latitude_south_ddeg {
                let mut longitude_nth = 0;
                let mut longitude_ddeg = longitude_west_ddeg;

                writeln!(
                    w,
                    "{}",
                    fmt_rinex(
                        &format!(
                            "  {:3.1}{:3.1}{:3.1}   {:3.1} {:3.1}",
                            latitude_ddeg,
                            longitude_east_ddeg,
                            longitude_west_ddeg,
                            longitude_spacing_ddeg,
                            altitude_km,
                        ),
                        "LAT/LON1/LON2/DLON/H"
                    )
                )?;

                while longitude_ddeg < longitude_east_ddeg {
                    let coords = QuantizedCoordinates::new(
                        latitude_ddeg,
                        -1,
                        longitude_ddeg,
                        -1,
                        altitude_km,
                        -1,
                    );

                    let key = IonexKey {
                        epoch: t,
                        coordinates: coords,
                    };

                    if let Some(tec) = record.get(&key) {
                        write!(w, "{:5}", tec.tecu())?;
                        has_rms |= tec.rms_tec().is_some();
                    } else {
                        write!(w, "9999 ")?;
                    }

                    longitude_ddeg += longitude_spacing_ddeg;

                    longitude_nth += 1;
                    longitude_nth = longitude_nth % NUM_LONGITUDES_PER_LINE;

                    if longitude_nth == 0 {
                        write!(w, "{}", '\n')?;
                    }
                }

                latitude_ddeg += latitude_spacing_ddeg;
            }

            altitude_km += altitude_spacing_km;
        }
        writeln!(w, "{}", fmt_rinex("", "END OF TEC MAP"))?;

        if has_rms {
            // format RMS map
            writeln!(
                w,
                "{}",
                fmt_rinex(&format!("{:6}", nth_map), "START OF RMS MAP")
            )?;

            while altitude_km < altitude_high_km {
                let mut latitude_ddeg = latitude_north_ddeg;

                while latitude_ddeg < latitude_south_ddeg {
                    let mut longitude_nth = 0;
                    let mut longitude_ddeg = longitude_west_ddeg;

                    writeln!(
                        w,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "  {:3.1}{:3.1}{:3.1}   {:3.1} {:3.1}",
                                latitude_ddeg,
                                longitude_east_ddeg,
                                longitude_west_ddeg,
                                longitude_spacing_ddeg,
                                altitude_km,
                            ),
                            "LAT/LON1/LON2/DLON/H"
                        )
                    )?;

                    while longitude_ddeg < longitude_east_ddeg {
                        let coords = QuantizedCoordinates::new(
                            latitude_ddeg,
                            -1,
                            longitude_ddeg,
                            -1,
                            altitude_km,
                            -1,
                        );

                        let key = IonexKey {
                            epoch: t,
                            coordinates: coords,
                        };

                        if let Some(tec) = record.get(&key) {
                            write!(w, "{:5}", tec.tecu())?;
                        } else {
                            write!(w, "9999 ")?;
                        }

                        longitude_ddeg += longitude_spacing_ddeg;

                        longitude_nth += 1;
                        longitude_nth = longitude_nth % NUM_LONGITUDES_PER_LINE;

                        if longitude_nth == 0 {
                            write!(w, "{}", '\n')?;
                        }
                    }

                    latitude_ddeg += latitude_spacing_ddeg;
                }

                altitude_km += altitude_spacing_km;
            }
            writeln!(w, "{}", fmt_rinex("", "END OF RMS MAP"))?;
        }

        if has_h {
            // format H map
            writeln!(
                w,
                "{}",
                fmt_rinex(&format!("{:6}", nth_map), "START OF HEIGHT MAP")
            )?;

            while altitude_km < altitude_high_km {
                let mut latitude_ddeg = latitude_north_ddeg;

                while latitude_ddeg < latitude_south_ddeg {
                    let mut longitude_nth = 0;
                    let mut longitude_ddeg = longitude_west_ddeg;

                    writeln!(
                        w,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "  {:3.1}{:3.1}{:3.1}   {:3.1} {:3.1}",
                                latitude_ddeg,
                                longitude_east_ddeg,
                                longitude_west_ddeg,
                                longitude_spacing_ddeg,
                                altitude_km,
                            ),
                            "LAT/LON1/LON2/DLON/H"
                        )
                    )?;

                    while longitude_ddeg < longitude_east_ddeg {
                        let coords = QuantizedCoordinates::new(
                            latitude_ddeg,
                            -1,
                            longitude_ddeg,
                            -1,
                            altitude_km,
                            -1,
                        );

                        let key = IonexKey {
                            epoch: t,
                            coordinates: coords,
                        };

                        if let Some(tec) = record.get(&key) {
                            write!(w, "{:5}", tec.tecu())?;
                        } else {
                            write!(w, "9999 ")?;
                        }

                        longitude_ddeg += longitude_spacing_ddeg;

                        longitude_nth += 1;
                        longitude_nth = longitude_nth % NUM_LONGITUDES_PER_LINE;

                        if longitude_nth == 0 {
                            write!(w, "{}", '\n')?;
                        }
                    }

                    latitude_ddeg += latitude_spacing_ddeg;
                }

                altitude_km += altitude_spacing_km;
            }
            writeln!(w, "{}", fmt_rinex("", "END OF HEIGHT MAP"))?;
        }
    }
    Ok(())
}
