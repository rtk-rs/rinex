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
                "     3.05           OBSERVATION DATA    GPS                 RINEX VERSION / TYPE",
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
