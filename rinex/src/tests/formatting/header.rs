use crate::{
    hardware::{Antenna, Receiver},
    prelude::{Constellation, Header, Version},
    tests::formatting::{generic_formatted_lines_test, Utf8Buffer},
};

use std::collections::HashMap;
use std::io::BufWriter;

#[test]
fn obs_header_formatting() {
    let mut buf = BufWriter::new(Utf8Buffer::new(1024));

    let header = Header::basic_obs()
        .with_version(Version::new(3, 5))
        .with_comment("test comment")
        .with_comment(
            "super long comment that needs to overflow the 60c limitation for testing purposes",
        )
        .with_general_information("TEST-PGM", "RUNBY", "AGENCY")
        .with_receiver(
            Receiver::default()
                .with_model("TEST RX")
                .with_serial_number("TEST SN-RX")
                .with_firmware("RX-FW"),
        )
        .with_receiver_antenna(
            Antenna::default()
                .with_model("TEST ANT")
                .with_serial_number("TEST SN-ANT"),
        )
        .with_constellation(Constellation::GPS);

    header.format(&mut buf).unwrap();

    let content = buf.into_inner().unwrap().to_ascii_utf8();

    generic_formatted_lines_test(
        &content,
        HashMap::from_iter([
            (
                0,
                "     3.05           OBSERVATION DATA    G (GPS)             RINEX VERSION / TYPE",
            ),
            (
                1,
                "TEST-PGM            RUNBY                                   PGM / RUN BY / DATE",
            ),
            (
                2,
                "                    AGENCY                                  OBSERVER / AGENCY",
            ),
            (
                3,
                "test comment                                                COMMENT",
            ),
            (
                4,
                "super long comment that needs to overflow the 60c limitationCOMMENT",
            ),
            (
                5,
                " for testing purposes                                       COMMENT",
            ),
            (
                6,
                "TEST SN-RX          TEST RX             RX-FW               REC # / TYPE / VERS",
            ),
            (
                7,
                "TEST SN-ANT         TEST ANT                                ANT # / TYPE",
            ),
            (
                8,
                "        0.0000        0.0000        0.0000                  ANTENNA: DELTA H/E/N",
            ),
            (
                9,
                "                                                            END OF HEADER",
            ),
        ]),
    );
}

#[test]
fn crinex_mixed_header_formatting() {
    let mut buf = BufWriter::new(Utf8Buffer::new(1024));

    let header = Header::basic_crinex();
    header.format(&mut buf).unwrap();

    let content = buf.into_inner().unwrap().to_ascii_utf8();

    generic_formatted_lines_test(
        &content,
        HashMap::from_iter([
            (
                0,
                "3.0                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE",
            ),
            (
                2,
                "     4.00           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE",
            ),
            (
                4,
                "                                                            END OF HEADER",
            ),
        ]),
    );
}

/// Formats the header of an IONEX file and checks the lines the
/// original file carries, then parses the result back.
#[cfg(feature = "flate2")]
fn ionex_header_round_trip(file: &str, expected_lines: &[&str]) {
    use crate::prelude::Rinex;
    use std::io::BufReader;

    let path = format!(
        "{}/../test_resources/IONEX/V1/{}",
        env!("CARGO_MANIFEST_DIR"),
        file
    );

    let header = Rinex::from_gzip_file(&path).unwrap().header;

    let mut buf = BufWriter::new(Utf8Buffer::new(4096));
    header.format(&mut buf).unwrap();
    let content = buf.into_inner().unwrap().to_ascii_utf8();

    for expected in expected_lines {
        assert!(
            content.lines().any(|line| line.trim_end() == *expected),
            "{}: missing line \"{}\" in\n{}",
            file,
            expected,
            content
        );
    }

    let parsed = Header::parse(&mut BufReader::new(content.as_bytes())).unwrap();

    assert_eq!(parsed.version, header.version);
    assert_eq!(parsed.rinex_type, header.rinex_type);
    assert_eq!(parsed.sampling_interval, header.sampling_interval);

    let (model, dut) = (header.ionex.unwrap(), parsed.ionex.unwrap());
    assert_eq!(dut.reference, model.reference);

    // the parser trims each description line: runs of blanks
    // at a wrapping point collapse to one
    let words = |description: &Option<String>| {
        description
            .as_ref()
            .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
    };
    assert_eq!(words(&dut.description), words(&model.description));
    assert_eq!(dut.epoch_of_first_map, model.epoch_of_first_map);
    assert_eq!(dut.epoch_of_last_map, model.epoch_of_last_map);
    assert_eq!(dut.number_of_maps, model.number_of_maps);
    assert_eq!(dut.mapping, model.mapping);
    assert_eq!(dut.elevation_cutoff, model.elevation_cutoff);
    assert_eq!(dut.observables, model.observables);
    assert_eq!(dut.nb_stations, model.nb_stations);
    assert_eq!(dut.nb_satellites, model.nb_satellites);
    assert_eq!(dut.base_radius, model.base_radius);
    assert_eq!(dut.grid, model.grid);
    assert_eq!(dut.exponent, model.exponent);
}

#[test]
#[cfg(feature = "flate2")]
fn ionex_header_formatting() {
    ionex_header_round_trip(
        "CKMG0020.22I.gz",
        &[
            "     1.0            IONOSPHERE MAPS     GNSS                IONEX VERSION / TYPE",
            "  2022     1     2     0     0     0                        EPOCH OF FIRST MAP",
            "  2022     1     3     0     0     0                        EPOCH OF LAST MAP",
            "  3600                                                      INTERVAL",
            "    25                                                      # OF MAPS IN FILE",
            "  NONE                                                      MAPPING FUNCTION",
            "     0.0                                                    ELEVATION CUTOFF",
            "  6371.0                                                    BASE RADIUS",
            "     2                                                      MAP DIMENSION",
            "   350.0 350.0   0.0                                        HGT1 / HGT2 / DHGT",
            "    87.5 -87.5  -2.5                                        LAT1 / LAT2 / DLAT",
            "  -180.0 180.0   5.0                                        LON1 / LON2 / DLON",
            "    -1                                                      EXPONENT",
            "                                                            END OF HEADER",
        ],
    );

    ionex_header_round_trip(
        "jplg0010.17i.gz",
        &[
            "     1.0            IONOSPHERE MAPS     GPS                 IONEX VERSION / TYPE",
            "  2017     1     1     0     0     0                        EPOCH OF FIRST MAP",
            "  2017     1     2     0     0     0                        EPOCH OF LAST MAP",
            "  7200                                                      INTERVAL",
            "    13                                                      # OF MAPS IN FILE",
            "    10.0                                                    ELEVATION CUTOFF",
            "One-way carrier phase leveled to code                       OBSERVABLES USED",
            "   170                                                      # OF STATIONS",
            "    31                                                      # OF SATELLITES",
            "   450.0 450.0   0.0                                        HGT1 / HGT2 / DHGT",
        ],
    );
}
