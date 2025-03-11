use crate::{
    doris::{DorisKey, Observations, SignalKey, SignalObservation, Station},
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    observation::ClockObservation,
    observation::EpochFlag,
    prelude::{Header, Observable, ParsingError, TimeScale},
};

/// Returns true if forwarded content does match new DORIS measurement.
pub fn is_new_epoch(line: &str) -> bool {
    line.starts_with('>')
}

fn parse_observations(
    line: &str,
    mut obs_ptr: usize,
    station: &Station,
    observables: &[Observable],
    numobs: usize,
    observations: &mut Observations,
) -> Result<(), ParsingError> {
    const OBSERVABLE_WIDTH: usize = 14;

    let mut offset = 0;
    let line_len = line.len();

    loop {
        let mut m1 = Option::<u8>::None;
        let mut m2 = Option::<u8>::None;

        if offset + OBSERVABLE_WIDTH + 1 < line_len {
            let slice = &line[offset + OBSERVABLE_WIDTH..offset + OBSERVABLE_WIDTH + 1];
            println!("flag : \"{}\"", slice);
            if let Ok(flag) = slice.trim().parse::<u8>() {
                m1 = Some(flag);
            }
        }

        if offset + OBSERVABLE_WIDTH + 2 < line_len {
            let slice = &line[offset + OBSERVABLE_WIDTH + 1..offset + OBSERVABLE_WIDTH + 2];
            println!("flag : \"{}\"", slice);
            if let Ok(flag) = slice.trim().parse::<u8>() {
                m2 = Some(flag);
            }
        }

        if offset + OBSERVABLE_WIDTH < line_len {
            let slice = &line[offset..offset + OBSERVABLE_WIDTH];
            println!("slice: \"{}\"", slice);
            if let Ok(value) = slice.trim().parse::<f64>() {
                let key = SignalKey {
                    station: station.clone(),
                    observable: observables[obs_ptr].clone(),
                };

                observations
                    .signals
                    .insert(key, SignalObservation { m1, m2, value });
            }
        }

        offset += OBSERVABLE_WIDTH;
        obs_ptr += 1;

        if obs_ptr == numobs {
            // abnormal content: abort and avoid overflowing
            break;
        }
    }

    Ok(())
}

/// Parse all DORIS measurements from forwarded content.
/// Needs reference to previously parsed [Header].
pub fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<(DorisKey, Observations), ParsingError> {
    const EPOCH_SIZE: usize = "YYYY MM DD HH MM SS.NNNNNNNNN  0".len();
    const CLOCK_OFFSET: usize = 26;
    const CLOCK_SIZE: usize = 14;
    const MIN_EPOCH_SIZE: usize = EPOCH_SIZE + CLOCK_SIZE + 2;

    let mut obs_ptr = 0;
    let flag = EpochFlag::default();

    let null_clock = ClockObservation::default();

    let mut station = Option::<Station>::None;
    let mut observations = Observations::default();

    let mut lines = content.lines();

    let doris = header
        .doris
        .as_ref()
        .ok_or(ParsingError::MissingObservableDefinition)?;

    let observables = &doris.observables;
    let numobs = observables.len();

    let stations = &doris.stations;

    // parse TAI timestamp
    let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
    let line_len = line.len();

    if line_len < MIN_EPOCH_SIZE {
        return Err(ParsingError::EpochFormat);
    }

    let epoch = parse_epoch_in_timescale(&line[2..2 + EPOCH_SIZE], TimeScale::TAI)?;

    let key = DorisKey { epoch, flag };

    // parse clock field
    let offset_s = line[CLOCK_OFFSET..CLOCK_OFFSET + CLOCK_SIZE]
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::DorisClockParsing)?;

    observations.clock = ClockObservation::with_offset_s(&null_clock, epoch, offset_s);

    // clock extrapolated ?
    observations.clock_extrapolated = false;

    if line_len > CLOCK_OFFSET + CLOCK_SIZE {
        if line[CLOCK_OFFSET + CLOCK_SIZE..].trim().eq("1") {
            observations.clock_extrapolated = true;
        }
    }

    // parse following stations
    for line in lines {
        if line.starts_with('D') {
            obs_ptr = 0;

            // station number
            let station_num = line[1..]
                .trim()
                .parse::<usize>()
                .map_err(|_| ParsingError::DorisStationFormat)?;

            // identify station
            if let Some(s) = stations.get(station_num) {
                station = Some(s.clone());
            }
        }

        if let Some(station) = &station {
            parse_observations(
                &line[4..],
                obs_ptr,
                &station,
                observables,
                numobs,
                &mut observations,
            )?;
        }
    }

    Ok((key, observations))
}

#[cfg(test)]
mod test {
    use super::is_new_epoch;

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

    //     #[test]
    //     fn valid_epoch() {
    //         let mut header = Header::default();
    //         let mut doris = DorisHeader::default();
    //         for obs in ["L1", "L2", "C1", "C2", "W1", "W2", "F", "P", "T", "H"] {
    //             let obs = Observable::from_str(obs).unwrap();
    //             doris.observables.push(obs);
    //         }
    //         for station in [
    //             "D01  THUB THULE                         43001S005  3   0",
    //             "D02  SVBC NY-ALESUND II                 10338S004  4   0",
    //         ] {
    //             let station = Station::from_str(station).unwrap();
    //             doris.stations.push(station);
    //         }
    //         header.doris = Some(doris);

    //         let content = "> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0
    // D01  -3237877.052    -2291024.044    21903595.62311  21903633.08011      -113.100 7
    //           -98.400 7       437.801        1002.000 1       -20.000 1        82.000 1
    // D02  -2069899.788     -407871.014     4677242.25714   4677392.20614      -119.050 7
    //          -111.000 7       437.801        1007.000 0        -2.000 0        74.000 0";

    //         let ((e, flag), content) =
    //             parse_epoch(&header, content).expect("failed to parse DORIS epoch");

    //         assert_eq!(
    //             e,
    //             Epoch::from_str("2024-01-01T00:00:28.999947700 TAI").unwrap(),
    //             "parsed wrong epoch"
    //         );
    //         assert_eq!(flag, EpochFlag::Ok, "parsed wrong epoch flag");

    //         let station = Station {
    //             key: 1,
    //             gen: 3,
    //             k_factor: 0,
    //             label: "THUB".to_string(),
    //             site: "THULE".to_string(),
    //             domes: DOMES {
    //                 site: 1,
    //                 area: 430,
    //                 sequential: 5,
    //                 point: DOMESTrackingPoint::Instrument,
    //             },
    //         };
    //         let values = content
    //             .get(&station)
    //             .unwrap_or_else(|| panic!("failed to identify {:?}", station));

    //         for (observable, data) in [
    //             (
    //                 Observable::from_str("L1C").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -3237877.052,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("L2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -2291024.044,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C1").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(1),
    //                     value: 21903595.623,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C2").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(1),
    //                     value: 21903633.080,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W1").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -113.100,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -98.400,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("F").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: 437.801,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("P").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(1),
    //                     value: 1002.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("T").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(1),
    //                     value: -20.0,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("H").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(1),
    //                     value: 82.0,
    //                 },
    //             ),
    //         ] {
    //             let value = values
    //                 .get(&observable)
    //                 .unwrap_or_else(|| panic!("failed to identify {:?}", observable));
    //             assert_eq!(value, &data, "wrong value parsed for {:?}", observable);
    //         }

    //         let station = Station {
    //             key: 2,
    //             gen: 4,
    //             k_factor: 0,
    //             label: "SVBC".to_string(),
    //             site: "NY-ALESUND II".to_string(),
    //             domes: DOMES {
    //                 site: 38,
    //                 area: 103,
    //                 sequential: 4,
    //                 point: DOMESTrackingPoint::Instrument,
    //             },
    //         };
    //         let values = content
    //             .get(&station)
    //             .unwrap_or_else(|| panic!("failed to identify {:?}", station));

    //         for (observable, data) in [
    //             (
    //                 Observable::from_str("L1C").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -2069899.788,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("L2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -407871.014,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C1").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(4),
    //                     value: 4677242.257,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C2").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(4),
    //                     value: 4677392.206,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W1").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -119.050,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -111.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("F").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: 437.801,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("P").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(0),
    //                     value: 1007.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("T").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(0),
    //                     value: -2.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("H").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(0),
    //                     value: 74.0,
    //                 },
    //             ),
    //         ] {
    //             let value = values
    //                 .get(&observable)
    //                 .unwrap_or_else(|| panic!("failed to identify {:?}", observable));
    //             assert_eq!(value, &data, "wrong value parsed for {:?}", observable);
    //         }
    //     }
}
