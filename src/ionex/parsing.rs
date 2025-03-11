//! IONEX maps parsing

use crate::{
    ionex::{IonexKey, Quantized, QuantizedCoordinates, Record, TEC},
    prelude::{Epoch, ParsingError},
};

pub fn is_new_tec_map(line: &str) -> bool {
    line.contains("START OF TEC MAP")
}

pub fn is_new_rms_map(line: &str) -> bool {
    line.contains("START OF RMS MAP")
}

pub fn is_new_height_map(line: &str) -> bool {
    line.contains("START OF HEIGHT MAP")
}

/// Parses Map grid to follow, returns
/// - fixed_latitude [ddeg]
/// - long1 [ddeg]
/// - long_spacing [ddeg]
/// - fixed_altitude [km]
fn parse_grid_specs(line: &str) -> Result<(f64, f64, f64, f64), ParsingError> {
    let (fixed_lat, rem) = line[2..].split_at(6);

    let fixed_lat = fixed_lat
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    let (long1, rem) = rem.split_at(6);

    let long1 = long1
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    // lon2 field is not used, we iterate using spacing starting @long1
    let (_, rem) = rem.split_at(6);

    let (long_spacing, rem) = rem.split_at(6);

    let long_spacing = long_spacing
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    let (fixed_alt, _) = rem.split_at(6);

    let fixed_alt = fixed_alt
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::IonexGridCoordinates)?;

    return Ok((fixed_lat, long1, long_spacing, fixed_alt));
}

/// Parses all maps contained in following TEC description.
/// This describes a serie of TEC point on an isosurface.
/// ## Inputs
///   - content: readable content (ASCII UTF-8)
///   - lat_exponent: deduced from IONEX header for coordinates quantization
///   - long_exponent: deduced from IONEX header for coordinates quantization
///   - tec_exponent: kept up to date, for correct data interpretation
///   - epoch: kept up to date, for correct classification
pub fn parse_tec_map(
    content: &str,
    lat_exponent: i8,
    long_exponent: i8,
    alt_exponent: i8,
    tec_exponent: i8,
    epoch: Epoch,
    record: &mut Record,
) -> Result<(), ParsingError> {
    const NON_AVAILABLE_TEC_KEYWORD: &str = "9999";
    let lines = content.lines();

    let mut fixed_lat = 0.0_f64;
    let mut long1;
    let mut long_spacing = 0.0_f64;
    let mut fixed_alt = 0.0_f64;

    let mut long = 0.0_f64; // current longitude (pointer)

    for line in lines {
        if line.len() > 60 {
            let (content, marker) = line.split_at(60);

            // Handle special cases
            // * data scaler update
            // * Timestamp specs (Epoch)
            if marker.contains("EXPONENT") {
                // should not have been presented (handled @ higher level)
                continue; // avoid parsing
            } else if marker.contains("EPOCH OF CURRENT MAP") {
                // should not have been presented (handled @ higher level)
                continue; // avoid parsing
            } else if marker.contains("START OF") {
                continue; // avoid parsing
            } else if marker.contains("LAT/LON1/LON2/DLON/H") {
                // grid specs (to follow)
                (fixed_lat, long1, long_spacing, fixed_alt) = parse_grid_specs(content)?;

                // determine quantization exponents

                long = long1; // reset pointer
                continue; // avoid parsing
            } else if marker.contains("END OF TEC MAP") {
                // block conclusion
                // don't care about block #id actually
                return Ok(());
            }
        }

        // proceed to parsing
        for item in line.split_ascii_whitespace() {
            // data interpretation
            let item = item.trim();
            if item != NON_AVAILABLE_TEC_KEYWORD {
                if let Ok(tecu) = item.parse::<i64>() {
                    let tec = TEC::from_quantized(tecu, tec_exponent);

                    let quantized_lat = Quantized::new(fixed_lat, lat_exponent);
                    let quantized_long = Quantized::new(long, long_exponent);
                    let quantized_alt = Quantized::new(fixed_alt, alt_exponent);

                    let coordinates = QuantizedCoordinates::from_quantized(
                        quantized_lat,
                        quantized_long,
                        quantized_alt,
                    );

                    let key = IonexKey { epoch, coordinates };

                    record.insert(key, tec);
                }
            }

            long += long_spacing;
        }
    }
    Ok(())
}

/// Parses all RMS maps contained in following content.
/// This describes the RMS value of each TEC previously parsed, for current isosurface.
/// ## Inputs
///   - content: readable content (ASCII UTF-8)
///   - lat_exponent: deduced from IONEX header for coordinates quantization
///   - long_exponent: deduced from IONEX header for coordinates quantization
///   - tec_exponent: kept up to date, for correct data interpretation
///   - epoch: epoch of current map
pub fn parse_rms_map(
    content: &str,
    lat_exponent: i8,
    long_exponent: i8,
    alt_exponent: i8,
    tec_exponent: i8,
    epoch: Epoch,
    record: &mut Record,
) -> Result<(), ParsingError> {
    const NON_AVAILABLE_TEC_KEYWORD: &str = "9999";

    let lines = content.lines();

    let mut fixed_lat = 0.0_f64;
    let mut fixed_alt = 0.0_f64;
    let mut long_spacing = 0.0_f64;

    let mut long = 0.0_f64; // current longitude (pointer)

    for line in lines {
        if line.len() > 60 {
            let marker = line.split_at(60).1;
            if marker.contains("END OF RMS MAP") {
                return Ok(());
            } else if marker.contains("EXPONENT") {
                // should not have been presented (handled @ higher level)
                continue; // avoid parsing
            } else if marker.contains("START OF") {
                continue; // avoid parsing
            } else if marker.contains("LAT/LON1/LON2/DLON/H") {
                // gric specs (to follow)
                (fixed_lat, long, long_spacing, fixed_alt) = parse_grid_specs(content)?;
                continue; // avoid parsing
            } else if marker.contains("END OF RMS MAP") {
                // block conclusion
                // don't care about block #id actually
                return Ok(());
            }
        }

        // proceed to parsing
        for item in line.split_ascii_whitespace() {
            let item = item.trim();

            if item != NON_AVAILABLE_TEC_KEYWORD {
                if let Ok(rms_tecu) = item.parse::<i64>() {
                    let quantized_lat = Quantized::new(fixed_lat, lat_exponent);
                    let quantized_long = Quantized::new(long, long_exponent);
                    let quantized_alt = Quantized::new(fixed_alt, alt_exponent);

                    let coordinates = QuantizedCoordinates::from_quantized(
                        quantized_lat,
                        quantized_long,
                        quantized_alt,
                    );

                    // we only augment previously parsed TEC values
                    let key = IonexKey { epoch, coordinates };
                    if let Some(v) = record.get_mut(&key) {
                        v.set_quantized_rms(rms_tecu, tec_exponent);
                    }
                }
            }

            long += long_spacing;
        }
    }
    Ok(())
}

// /// Parses all Height map contained in following content.
// /// Adjust the previously parsed isosurface to turn them into a real volume definition.
// /// ## Inputs
// ///   - content: readable content (ASCII UTF-8)
// ///   - lat_exponent: deduced from IONEX header for coordinates quantization
// ///   - long_exponent: deduced from IONEX header for coordinates quantization
// ///   - tec_exponent: kept up to date, for correct data interpretation
// ///   - epoch: epoch of current map
// pub fn parse_height_map(
//     content: &str,
//     lat_exponent: i8,
//     long_exponent: i8,
//     alt_exponent: i8,
//     tec_exponent: i8,
//     epoch: Epoch,
//     record: &mut Record,
// ) -> Result<(), ParsingError> {
//     let lines = content.lines();
//     let mut epoch = Epoch::default();

//     let mut fixed_lat = 0.0_f64;
//     let (mut long1, mut long_spacing) = (0.0_f64, 0.0_f64);
//     let mut fixed_alt = 0.0_f64;

//     let mut long = 0.0_f64; // current longitude (pointer)

//     for line in lines {
//         if line.len() > 60 {
//             let marker = line.split_at(60).1;
//             if marker.contains("END OF HEIGHT MAP") {
//                 return Ok(());
//             } else if marker.contains("EXPONENT") {
//                 // should not have been presented (handled @ higher level)
//                 continue; // avoid parsing
//             }
//         }

//         // proceed to parsing
//         for item in line.split_ascii_whitespace() {
//             if let Ok(h_km) = item.trim().parse::<i32>() {
//                 // Should adjust the altitude we previously parsed
//             }

//             long += long_spacing;
//         }
//     }
//     Ok(())
// }

#[cfg(test)]
mod test {
    use super::{
        is_new_height_map, is_new_rms_map, is_new_tec_map, parse_grid_specs, parse_tec_map,
        Quantized,
    };

    use crate::{
        ionex::{IonexKey, QuantizedCoordinates, Record},
        prelude::Epoch,
    };

    #[test]
    fn new_ionex_map() {
        assert!(is_new_tec_map(
            "1                                                      START OF TEC MAP"
        ));

        assert!(is_new_rms_map(
            "1                                                      START OF RMS MAP"
        ));

        assert!(is_new_height_map(
            "1                                                   START OF HEIGHT MAP"
        ));
    }

    #[test]
    fn grid_specs_parsing() {
        let content =
            "    87.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H";

        let (fixed_lat, long1, long_spacing, fixed_alt) = parse_grid_specs(content).unwrap();
        assert_eq!(fixed_lat, 87.5);
        assert_eq!(long1, -180.0);
        assert_eq!(long_spacing, 5.0);
        assert_eq!(fixed_alt, 450.0);

        let content =
            "     2.5-180.0 180.0   5.0 350.0                            LAT/LON1/LON2/DLON/H";

        let (fixed_lat, long1, long_spacing, fixed_alt) = parse_grid_specs(content).unwrap();
        assert_eq!(fixed_lat, 2.5);
        assert_eq!(long1, -180.0);
        assert_eq!(long_spacing, 5.0);
        assert_eq!(fixed_alt, 350.0);

        let content =
            "    -2.5-180.0 180.0   5.0 250.0                            LAT/LON1/LON2/DLON/H";

        let (fixed_lat, long1, long_spacing, fixed_alt) = parse_grid_specs(content).unwrap();
        assert_eq!(fixed_lat, -2.5);
        assert_eq!(long1, -180.0);
        assert_eq!(long_spacing, 5.0);
        assert_eq!(fixed_alt, 250.0);
    }

    #[test]
    fn tec_map_parsing() {
        let mut record = Record::default();

        let lat_exponent = Quantized::find_exponent(2.5);
        let long_exponent = Quantized::find_exponent(5.0);
        let alt_exponent = Quantized::find_exponent(0.0);
        let tec_exponent = -1;
        let epoch = Epoch::from_gregorian_utc_at_midnight(2017, 1, 1);

        let content =
            "     1                                                      START OF TEC MAP    
  2017     1     1     0     0     0                        EPOCH OF CURRENT MAP
    87.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
   33   33   32   32   32   31   31   30   30   30   29   29   28   28   28   27
   27   27   26   26   26   26   26   26   26   26   26   26   26   26   26   26
   27   27   27   28   28   29   29   30   30   31   31   32   32   33   33   33
   34   34   35   35   35   35   36   36   36   36   36   36   36   36   36   35
   35   35   35   35   34   34   34   33   33
    85.0-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
   36   36   35   35   34   34   33   33   32   31   31   30   29   28   28   27
   26   25   25   24   24   23   23   22   22   22   22   22   22   23   23   24
   24   25   25   26   27   28   29   29   30   31   32   33   34   35   36   37
   38   39   39   40   41   41   41   41   42   42   42   41   41   41   41   40
   40   40   39   39   38   38   37   37   36
    27.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
   235  230  222  212  200  187  173  157  141  126  110   95   92   92   92   92
    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    92   92   92   92   92   92   92   92   92  104  120  136  151  166  180  193
   205  215  224  231  236  239  240  239  235
     2.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
   364  370  374  378  380  380  378  375  370  364  356  346  336  324  311  298
   283  269  253  238  222  207  191  175  159  143  127  111   96   92   92   92
    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    92   92   92   92   92  106  124  141  158  175  191  207  223  238  252  266
   280  293  305  317  328  339  348  356  364
    -2.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
   363  370  375  380  383  385  385  384  381  376  370  363  354  343  332  319
   305  291  276  260  244  227  210  194  176  159  143  126  109   93   92   92
    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    92   92   92   92  103  120  136  152  168  183  198  212  226  240  253  266
   279  291  303  315  326  336  346  355  363
     1                                                      END OF TEC MAP      ";

        parse_tec_map(
            content,
            lat_exponent,
            long_exponent,
            alt_exponent,
            tec_exponent,
            epoch,
            &mut record,
        )
        .unwrap();

        for (coordinates, quantized_tecu) in [
            (
                QuantizedCoordinates::new(
                    87.5,
                    lat_exponent,
                    -180.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                33,
            ),
            (
                QuantizedCoordinates::new(
                    87.5,
                    lat_exponent,
                    -175.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                33,
            ),
            (
                QuantizedCoordinates::new(
                    87.5,
                    lat_exponent,
                    -170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                32,
            ),
            (
                QuantizedCoordinates::new(
                    87.5,
                    lat_exponent,
                    170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                34,
            ),
            (
                QuantizedCoordinates::new(
                    87.5,
                    lat_exponent,
                    175.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                33,
            ),
            (
                QuantizedCoordinates::new(
                    87.5,
                    lat_exponent,
                    180.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                33,
            ),
            (
                QuantizedCoordinates::new(
                    85.0,
                    lat_exponent,
                    -180.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                36,
            ),
            (
                QuantizedCoordinates::new(
                    85.0,
                    lat_exponent,
                    -175.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                36,
            ),
            (
                QuantizedCoordinates::new(
                    85.0,
                    lat_exponent,
                    -170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                35,
            ),
            (
                QuantizedCoordinates::new(
                    85.0,
                    lat_exponent,
                    170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                37,
            ),
            (
                QuantizedCoordinates::new(
                    85.0,
                    lat_exponent,
                    175.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                37,
            ),
            (
                QuantizedCoordinates::new(
                    85.0,
                    lat_exponent,
                    180.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                36,
            ),
            (
                QuantizedCoordinates::new(
                    27.5,
                    lat_exponent,
                    170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                240,
            ),
            (
                QuantizedCoordinates::new(
                    27.5,
                    lat_exponent,
                    175.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                239,
            ),
            (
                QuantizedCoordinates::new(
                    27.5,
                    lat_exponent,
                    180.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                235,
            ),
            (
                QuantizedCoordinates::new(
                    2.5,
                    lat_exponent,
                    -170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                374,
            ),
            (
                QuantizedCoordinates::new(
                    2.5,
                    lat_exponent,
                    170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                348,
            ),
            (
                QuantizedCoordinates::new(
                    2.5,
                    lat_exponent,
                    175.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                356,
            ),
            (
                QuantizedCoordinates::new(
                    2.5,
                    lat_exponent,
                    180.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                364,
            ),
            (
                QuantizedCoordinates::new(
                    -2.5,
                    lat_exponent,
                    -170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                375,
            ),
            (
                QuantizedCoordinates::new(
                    -2.5,
                    lat_exponent,
                    170.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                346,
            ),
            (
                QuantizedCoordinates::new(
                    -2.5,
                    lat_exponent,
                    175.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                355,
            ),
            (
                QuantizedCoordinates::new(
                    -2.5,
                    lat_exponent,
                    180.0,
                    long_exponent,
                    450.0,
                    alt_exponent,
                ),
                363,
            ),
        ] {
            let key = IonexKey { epoch, coordinates };

            let tec = record
                .get(&key)
                .expect(&format!("missing value at {:#?}", key));

            let tecu = tec.tecu();
            let expected = quantized_tecu as f64 * 10.0_f64.powi(tec_exponent as i32);
            let err = (tecu - expected).abs();
            assert!(err < 1.0E-6);
        }
    }
}
