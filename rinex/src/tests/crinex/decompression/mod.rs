// V1 test
mod ajac3550;
// V1 very old
mod kosg0010;
// V3 test
mod esbcdnk;
// v3 test
mod pdel0010_21;
// v1 test with clock (not very meaningful though..)
mod vlns0630;

use std::{
    collections::HashMap,
    str::{from_utf8, FromStr},
};

use crate::{
    hatanaka::Decompressor,
    prelude::{Constellation, Observable},
};

/// This method is used by all "raw" decompression tests
/// to compare the decompressed content to what we expect
/// Inputs:
/// - v3 (boolean)
/// - constellation: as defined in header
/// - constellation (from supposedly parsed header)
/// - specs (from spposedly parsed header)
pub fn run_raw_decompression_test(
    v3: bool,
    constellation: &str,
    constellation_specs: &[&str],
    observable_specs: &[&str],
    input: &str,
    output: &str,
) {
    let constellation = Constellation::from_str(constellation.trim()).unwrap();

    let mut gnss_observables = HashMap::<Constellation, Vec<Observable>>::new();

    for (constell, observables) in constellation_specs.iter().zip(observable_specs.iter()) {
        let constell = Constellation::from_str(constell.trim()).unwrap();
        let observables = observables
            .split(',')
            .map(|ob| Observable::from_str(ob.trim()).unwrap())
            .collect::<Vec<_>>();
        gnss_observables.insert(constell, observables);
    }

    // build decompressor
    let mut decomp = Decompressor::new(v3, constellation, gnss_observables);

    // run test, on every provided input versus output
    let input_lines = input.lines();
    let mut output_lines = output.lines();

    let mut buf = [0; 4096];

    for (nth, input) in input_lines.enumerate() {
        // decompress each input
        match decomp.decompress(input, input.len(), &mut buf, 4096) {
            Ok(size) => {
                // for each generated line (may have several in V1)
                // we compare to expected outputs
                let content = from_utf8(&buf[..size])
                    .expect("CRNX2RNX should always produce valid UTF-8")
                    .to_string();

                let generated_lines = content.lines();
                let len = generated_lines.clone().count();

                if len < 2 {
                    // we are not 100 % equivalent, in terms of trailing whitespace
                    let content = content.trim_end();

                    // expected content may omit trailing blank lines
                    let output = output_lines.next().unwrap_or("").trim_end();

                    assert_eq!(content, output, "failed on line={} \"{}\"", nth, input);
                } else {
                    // in V1 we may generate more than 1 line for 1 input line
                    for line in generated_lines {
                        // we are not 100 % equivalent, in terms of trailing whitespace
                        let content = line.trim_end();

                        // expected content may omit trailing blank lines
                        let output = output_lines.next().unwrap_or("").trim_end();

                        assert_eq!(content, output, "failed on line={} \"{}\"", nth, input);
                    }
                }
            },
            Err(e) => panic!("decompression failed with {}", e),
        }
    }
}

use crate::{
    observation::EpochFlag,
    prelude::{Epoch, GeodeticMarker, MarkerType, Rinex, SV},
    tests::toolkit::{
        generic_observation_comparison, generic_observation_rinex_test, random_name,
        ClockDataPoint, SignalDataPoint, TimeFrame,
    },
};

use std::{fs::remove_file as fs_remove_file, io::BufReader, path::Path};

#[test]
fn testbench_v1() {
    let pool = vec![
        ("zegv0010.21d", "zegv0010.21o"),
        ("AJAC3550.21D", "AJAC3550.21O"),
        //("KOSG0010.95D", "KOSG0010.95O"), //TODO@ fix tests/obs/v2_kosg first
        ("aopr0010.17d", "aopr0010.17o"),
        ("npaz3550.21d", "npaz3550.21o"),
        ("wsra0010.21d", "wsra0010.21o"),
    ];
    for (crnx_name, rnx_name) in pool {
        // parse DUT
        let path = format!("../test_resources/CRNX/V1/{}", crnx_name);
        let crnx = Rinex::from_file(&path);

        assert!(crnx.is_ok(), "failed to parse {}", path);
        let mut dut = crnx.unwrap();

        let header = dut.header.obs.as_ref().unwrap();

        assert!(header.crinex.is_some());
        let infos = header.crinex.as_ref().unwrap();

        if crnx_name.eq("zegv0010.21d") {
            assert_eq!(infos.version.major, 1);
            assert_eq!(infos.version.minor, 0);
            assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
            assert_eq!(
                infos.date,
                Epoch::from_gregorian_utc(2021, 01, 02, 00, 01, 00, 00)
            );
            generic_observation_rinex_test(
                    &dut,
                    "2.11",
                    Some("MIXED"),
                    false,
                    "G07, G08, G10, G13, G15, G16, G18, G20, G21, G23, G26, G27, G30, R01, R02, R03, R08, R09, R15, R16, R17, R18, R19, R24",
                    "GPS, GLO",
                    &[
                        ("GPS", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5"),
                        ("GLO", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5"),
                    ],
                    Some("2021-01-01T00:00:00 GPST"),
                    Some("2021-01-01T23:59:30 GPST"),
                    None,
                    None,
                    None,
                    TimeFrame::from_inclusive_csv("2021-01-01T00:00:00 GPST, 2021-01-01T00:09:00 GPST, 30 s"),
                    vec![],
                    vec![],
                );
        } else if crnx_name.eq("npaz3550.21d") {
            assert_eq!(infos.version.major, 1);
            assert_eq!(infos.version.minor, 0);
            assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
            assert_eq!(
                infos.date,
                Epoch::from_gregorian_utc(2021, 12, 28, 00, 18, 00, 00)
            );
            generic_observation_rinex_test(
                &dut,
                "2.11",
                Some("MIXED"),
                false,
                "G01, G08, G10, G15, G16, G18, G21, G23, G26, G32, R04, R05, R06, R07, R10, R12, R19, R20, R21, R22",
                "GPS, GLO",
                &[("GPS", "C1, L1, L2, P2, S1, S2")],
                Some("2021-12-21T00:00:00 GPST"),
                Some("2021-12-21T23:59:30 GPST"),
                None,
                None,
                None,
                TimeFrame::from_inclusive_csv(
                    "2021-12-21T00:00:00 GPST, 2021-12-21T01:04:00 GPST, 30 s",
                ),
                vec![],
                vec![],
            );
        } else if crnx_name.eq("wsra0010.21d") {
            generic_observation_rinex_test(
                    &dut,
                    "2.11",
                    Some("MIXED"),
                    false,
                    "R09, R02, G07, G13, R17, R16, R01, G18, G26, G10, G30, G23, G27, G08, R18, G20, R15, G21, G15, R24, G16",
                    "GPS, GLO",
                    &[
                        ("GPS", "L1, L2, C1, P2, P1, S1, S2"),
                        ("GPS", "L1, L2, C1, P2, P1, S1, S2"),
                    ],
                    Some("2021-01-01T00:00:00 GPST"),
                    None,
                    None,
                    None,
                    None,
                    TimeFrame::from_inclusive_csv("2021-01-01T00:00:00 GPST, 2021-01-01T00:08:00 GPST, 30 s"),
                    vec![],
                    vec![],
                );
        } else if crnx_name.eq("aopr0010.17d") {
            generic_observation_rinex_test(
                &dut,
                "2.10",
                Some("GPS"),
                false,
                "G01, G03, G06, G07, G08, G09, G11, G14, G16, G17, G19, G22, G23, G26, G27, G28, G30, G31, G32",
                "GPS",
                &[("GPS", "C1, L1, L2, P1, P2")],
                Some("2017-01-01T00:00:00 GPST"),
                None,
                None,
                None,
                None,
                TimeFrame::from_erratic_csv(
                    "
                    2017-01-01T00:00:00 GPST,
                    2017-01-01T03:33:40 GPST,
                    2017-01-01T06:09:10 GPST
                    ",
                ),
                vec![],
                vec![],
            );
        //} else if crnx_name.eq("KOSG0010.95D") {
        //    test_observation_rinex(
        //        &rnx,
        //        "2.0",
        //        Some("GPS"),
        //        "GPS",
        //        "G01, G04, G05, G06, G16, G17, G18, G19, G20, G21, G22, G23, G24, G25, G27, G29, G31",
        //        "C1, L1, L2, P2, S1",
        //        Some("1995-01-01T00:00:00 GPST"),
        //        Some("1995-01-01T23:59:30 GPST"),
        //        erratic_time_frame!("
        //            1995-01-01T00:00:00 GPST,
        //            1995-01-01T11:00:00 GPST,
        //            1995-01-01T20:44:30 GPST
        //        "),
        //    );
        } else if crnx_name.eq("AJAC3550.21D") {
            generic_observation_rinex_test(
                    &dut,
                    "2.11",
                    Some("MIXED"),
                    false,
                    "G07, G08, G10, G16, G18, G21, G23, G26, G32, R04, R05, R10, R12, R19, R20, R21, E04, E11, E12, E19, E24, E25, E31, E33, S23, S36",
                    "GPS, GLO, GAL, EGNOS",
                    &[
                        ("GPS", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                        ("GLO", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                        ("GAL", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                        ("SBAS", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                    ],
                    Some("2021-12-21T00:00:00 GPST"),
                    None,
                    None,
                    None,
                    None,
                    TimeFrame::from_inclusive_csv("2021-12-21T00:00:00 GPST, 2021-12-21T00:00:30 GPST, 30 s"),
                    vec![],
                    vec![],
                );
        }

        // decompress and write to file
        dut.crnx2rnx_mut();

        let specs = dut.header.obs.as_ref().unwrap();
        assert!(specs.crinex.is_none());

        // TODO
        let filename = format!("{}.rnx", random_name(10));

        // assert!(
        //     dut.to_file(&filename).is_ok(),
        //     "failed to dump \"{}\" after decompression",
        //     crnx_name
        // );

        // run test on generated file
        let path = format!("../test_resources/OBS/V2/{}", rnx_name);
        let _model = Rinex::from_file(&path).unwrap();

        // TODO unlock this
        // generic_observation_rinex_against_model();

        let _ = fs_remove_file(filename); // cleanup
    }
}
#[test]
fn testbench_v3() {
    let pool = vec![
        ("DUTH0630.22D", "DUTH0630.22O"),
        //(
        //    "ACOR00ESP_R_20213550000_01D_30S_MO.crx",
        //    "ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
        //),
        //("pdel0010.21d", "pdel0010.21o"),
        //("flrs0010.12d", "flrs0010.12o"),
        //("VLNS0010.22D", "VLNS0010.22O"),
        //("VLNS0630.22D", "VLNS0630.22O"),
        //TODO unlock this
        //("ESBC00DNK_R_20201770000_01D_30S_MO.crx", "ESBC00DNK_R_20201770000_01D_30S_MO.rnx"),
        //("KMS300DNK_R_20221591000_01H_30S_MO.crx", "KMS300DNK_R_20221591000_01H_30S_MO.rnx"),
        //("MOJN00DNK_R_20201770000_01D_30S_MO.crx", "MOJN00DNK_R_20201770000_01D_30S_MO.rnx"),
    ];
    for (crnx_name, rnx_name) in pool {
        // parse DUT
        let path = format!("../test_resources/CRNX/V3/{}", crnx_name);

        let mut dut = Rinex::from_file(&path).unwrap();

        assert!(dut.header.obs.is_some());
        let obs = dut.header.obs.as_ref().unwrap();

        assert!(obs.crinex.is_some());
        let infos = obs.crinex.as_ref().unwrap();

        if crnx_name.eq("DUTH0630.22D") {
            assert_eq!(infos.version.major, 3);
            assert_eq!(infos.version.minor, 0);
            assert_eq!(infos.prog, "RNX2CRX ver.4.1.0");

            generic_observation_rinex_test(
                &dut,
                "3.02",
                Some("MIXED"),
                false, // has_clock
                "G01, G03, G04, G06, G09, G17, G19, G21, G22, G26, G31, G32, R01, R02, R08, R09, R10, R17, R23, R24",
                "GPS, GLO",
                &[
                    ("GPS", "C1C, L1C, D1C, S1C, C2W, L2W, D2W, S2W"),
                    ("GLO", "C1C, L1C, D1C, S1C, C2P, L2P, D2P, S2P"),
                ],
                Some("2022-03-04T00:00:00 GPST"),
                Some("2022-03-04T23:59:30 GPST"),
                None, // ground_ref_wgs84_m
                None, // observer
                None, // geodetic_marker
                TimeFrame::from_erratic_csv(
                    "2022-03-04T00:00:00 GPST, 
                    2022-03-04T00:28:30 GPST,
                    2022-03-04T00:57:00 GPST"),
                vec![], // signals
                vec![], // clocks
            );
        } else if crnx_name.eq("ACOR00ESP_R_20213550000_01D_30S_MO.crx") {
            assert_eq!(infos.version.major, 3);
            assert_eq!(infos.version.minor, 0);
            assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
            assert_eq!(
                infos.date,
                Epoch::from_gregorian_utc(2021, 12, 28, 01, 01, 00, 00)
            );

            generic_observation_rinex_test(
                &dut,
                "3.04",
                Some("MIXED"),
                false, // has_clock
                "GPS, GLO, GAL, BDS",
                "G01, G07, G08, G10, G16, G18, G21, G23, G26, G30, R04, R05, R10, R12, R20, R21, E02, E11, E12, E24, E25, E31, E33, E36, C05, C11, C14, C21, C22, C23, C25, C28, C34, C37, C42, C43, C44, C58",
                &[
                    ("GPS", "C1C, L1C, S1C, C2S, L2S, S2S, C2W, L2W, S2W, C5Q, L5Q, S5Q"),
                    ("GLO", "C1C, L1C, S1C, C2P, L2P, S2P, C2C, L2C, S2C, C3Q, L3Q, S3Q"),
                    ("GAL", "C1C, L1C, S1C, C5Q, L5Q, S5Q, C6C, L6C, S6C, C7Q, L7Q, S7Q, C8Q, L8Q, S8Q"),
                    ("BDS", "C2I, L2I, S2I, C6I, L6I, S6I, C7I, L7I, S7I"),
                ],
                Some("2021-12-21T00:00:00 GPST"),
                Some("2021-12-21T23:59:30 GPST"),
                None, // ground_ref_wgs84_m
                None, // observer
                None, // geodetic_marker
                TimeFrame::from_inclusive_csv("2021-12-21T00:00:00 GPST, 2021-12-21T00:12:00 GPST, 30 s"),
                vec![], // signals
                vec![], // clocks
            );
        }

        // convert to RINEX
        dut.crnx2rnx_mut();

        let obs = dut.header.obs.as_ref().unwrap();
        assert!(obs.crinex.is_none());

        // run test on generated file
        let path = format!("../test_resources/OBS/V3/{}", rnx_name);
        let model = Rinex::from_file(&path).unwrap();

        // TODO unlock this
        // generic_observation_rinex_against_model();
    }
}

/// Compares every receiver clock offset decompressed from `crnx_path`
/// against the RINEX model at `rnx_path` (produced by the historical CRX2RNX tool),
/// then verifies a few hand-picked `points` (epoch, offset in seconds).
fn run_clock_offsets_test(
    crnx_path: &str,
    rnx_path: &str,
    expected_epochs: usize,
    compare_signals: bool,
    points: &[(&str, f64)],
) {
    let crnx_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join(crnx_path);

    let rnx_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join(rnx_path);

    let dut = Rinex::from_file(crnx_path.to_string_lossy().as_ref()).unwrap();
    let model = Rinex::from_file(rnx_path.to_string_lossy().as_ref()).unwrap();

    if compare_signals {
        // strict point by point comparison of every signal (and clock)
        generic_observation_comparison(&dut, &model);
    }

    let dut_epochs = dut.epoch_iter().count();
    assert_eq!(dut_epochs, expected_epochs, "wrong number of epochs");
    assert_eq!(dut_epochs, model.epoch_iter().count());

    let dut_clocks = dut.clock_observations_iter().collect::<Vec<_>>();
    let model_clocks = model.clock_observations_iter().collect::<Vec<_>>();

    assert!(!model_clocks.is_empty(), "model has no clock offsets");
    assert_eq!(
        dut_clocks.len(),
        model_clocks.len(),
        "wrong number of clock offsets"
    );

    for ((dut_k, dut_clock), (model_k, model_clock)) in dut_clocks.iter().zip(model_clocks.iter()) {
        assert_eq!(dut_k, model_k, "clock offset attached to wrong epoch");
        assert_eq!(
            dut_clock.offset_s, model_clock.offset_s,
            "wrong clock offset at {}",
            dut_k.epoch
        );
    }

    for (epoch, offset_s) in points {
        let epoch = Epoch::from_str(epoch).unwrap();
        let (_, clock) = dut_clocks
            .iter()
            .find(|(k, _)| k.epoch == epoch)
            .unwrap_or_else(|| panic!("missing clock offset at {}", epoch));
        assert_eq!(clock.offset_s, *offset_s, "wrong clock offset at {}", epoch);
    }
}

/// Regression test for <https://github.com/nav-solutions/rinex/issues/426>.
/// One hour of the BKG ACRG00GHA_R_20240010000_01D_30S_MO file reduced to a single SV,
/// compressed with RNX2CRX 4.1.0. The receiver clock offset is reset ("3&") at
/// order 3, then compressed, omitted for a few epochs, then reset five more times.
/// On the released kernel this sequence overflows at epoch 91: the test exercises
/// both symptoms of the issue, the panic and the silently missing clock offsets.
/// The model is the CRX2RNX 4.1.0 output.
#[test]
fn v3_acrg00gha_clock_offsets() {
    run_clock_offsets_test(
        "CRNX/V3/ACRG00GHA_R_20240010000_01H_30S_MO.crx",
        "OBS/V3/ACRG00GHA_R_20240010000_01H_30S_MO.rnx",
        120,
        true, // signals strictly compared with the CRX2RNX model
        &[
            // kernel reset: 3&1520
            ("2024-01-01T00:00:00 GPST", 0.000000001520),
            // first compressed value (order 1)
            ("2024-01-01T00:00:30 GPST", 0.000000003404),
            // steady state (order 3)
            ("2024-01-01T00:05:00 GPST", 0.000000001788),
            // last value before the clock is omitted
            ("2024-01-01T00:15:00 GPST", 0.000000000204),
            // kernel reset after the gap: 3&-1459
            ("2024-01-01T00:20:00 GPST", -0.000000001459),
        ],
    );

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("CRNX")
        .join("V3")
        .join("ACRG00GHA_R_20240010000_01H_30S_MO.crx");

    let dut = Rinex::from_file(path.to_string_lossy().as_ref()).unwrap();

    generic_observation_rinex_test(
        &dut,
        "3.05",
        Some("MIXED"),
        true, // has_clock
        "I02",
        "IRNSS",
        &[("IRNSS", "C5A, D5A, L5A, S5A")],
        Some("2024-01-01T00:00:00 GPST"),
        None,
        None, // ground_ref_wgs84_m
        None, // observer
        None, // geodetic_marker
        TimeFrame::from_inclusive_csv("2024-01-01T00:00:00 GPST, 2024-01-01T00:59:30 GPST, 30 s"),
        vec![], // signals
        vec![
            // kernel reset: 3&1520
            ClockDataPoint::new(
                Epoch::from_str("2024-01-01T00:00:00 GPST").unwrap(),
                EpochFlag::Ok,
                0.000000001520,
            ),
            // first compressed value (order 1)
            ClockDataPoint::new(
                Epoch::from_str("2024-01-01T00:00:30 GPST").unwrap(),
                EpochFlag::Ok,
                0.000000003404,
            ),
            // steady state (order 3)
            ClockDataPoint::new(
                Epoch::from_str("2024-01-01T00:05:00 GPST").unwrap(),
                EpochFlag::Ok,
                0.000000001788,
            ),
            // last value before the clock is omitted
            ClockDataPoint::new(
                Epoch::from_str("2024-01-01T00:15:00 GPST").unwrap(),
                EpochFlag::Ok,
                0.000000000204,
            ),
            // kernel reset after the gap: 3&-1459
            ClockDataPoint::new(
                Epoch::from_str("2024-01-01T00:20:00 GPST").unwrap(),
                EpochFlag::Ok,
                -0.000000001459,
            ),
        ],
    );
}

/// RINEX 2.11 counterpart of the previous test: a delf0010.21o extract with
/// receiver clock offsets (F12.9, columns 69-80), compressed with RNX2CRX 4.1.0.
/// Includes a gap in the clock reports and a clock jump. The model is the
/// CRX2RNX 4.1.0 output.
/// Only the clock offsets are verified here: V1 observation decompression
/// is covered (and currently failing) in `testbench_v1`.
#[test]
fn v1_delf0010_clock_offsets() {
    run_clock_offsets_test(
        "CRNX/V1/delf0010_clock.21d",
        "OBS/V2/delf0010_clock.21o",
        24,
        true, // signals strictly compared with the CRX2RNX model
        &[
            // kernel reset: 3&-123456
            ("2021-01-01T00:00:00 GPST", -0.000123456),
            // 2-character compressed value: "36"
            ("2021-01-01T00:00:30 GPST", -0.000123420),
            // last value before the gap
            ("2021-01-01T00:04:30 GPST", -0.000123204),
            // kernel reset after the gap
            ("2021-01-01T00:07:00 GPST", -0.000123134),
            // clock jump
            ("2021-01-01T00:09:00 GPST", 0.000001131),
        ],
    );
}

/// gh-426: a numerical state that no longer fits in an i64 must end the
/// parsing with what was recovered so far, not panic (debug) nor wrap
/// silently (release). The first kernel reset of the ACRG00GHA extract is
/// replaced by i64::MAX, so the very next compressed value overflows.
#[test]
fn v3_clock_overflow_does_not_panic() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("CRNX")
        .join("V3")
        .join("ACRG00GHA_R_20240010000_01H_30S_MO.crx");

    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content.matches("3&1520\n").count(), 1);
    let corrupted = content.replace("3&1520\n", "3&9223372036854775807\n");

    let mut reader = BufReader::new(corrupted.as_bytes());
    let rinex = Rinex::parse(&mut reader).unwrap();

    // parsing stops at the corrupted epoch: at most the epoch carrying
    // the reset itself is recovered, nothing past it
    assert!(rinex.epoch_iter().count() <= 1);
    for (_, clock) in rinex.clock_observations_iter() {
        assert_eq!(clock.offset_s, i64::MAX as f64 / 1.0E12);
    }
}

#[test]
fn v1_zegv0010_21d() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("CRNX")
        .join("V1")
        .join("zegv0010.21d");

    let fullpath = path.to_string_lossy();
    let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

    generic_observation_rinex_test(
            &dut,
            "2.11",
            Some("MIXED"),
            false,
            "G07, G08, G10, G13, G15, G16, G18, G20, G21, G23, G26, G27, G30, R01, R02, R03, R08, R09, R15, R16, R17, R18, R19, R24",
            "GPS, GLO",
            &[
                ("GPS", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5"),
            ],
            Some("2021-01-01T00:00:00 GPST"),
            Some("2021-01-01T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2021-01-01T00:00:00 GPST, 2021-01-01T00:09:00 GPST, 30 s"),
            vec![
                SignalDataPoint::new(
                    Epoch::from_str("2021-01-01T00:00:00 GPST").unwrap(),
                    EpochFlag::Ok,
                    SV::from_str("G07").unwrap(),
                    Observable::from_str("C1").unwrap(),
                    24178026.635,
                ),
            ],
            vec![],
        );
}

#[test]
fn v3_acor00esp_r2021() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("CRNX")
        .join("V3")
        .join("ACOR00ESP_R_20213550000_01D_30S_MO.crx");

    let fullpath = path.to_string_lossy();
    let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

    assert!(dut.header.obs.is_some());
    let obs = dut.header.obs.as_ref().unwrap();
    assert!(obs.crinex.is_some());

    let infos = obs.crinex.as_ref().unwrap();

    assert_eq!(infos.version.major, 3);
    assert_eq!(infos.version.minor, 0);
    assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
    assert_eq!(
        infos.date,
        Epoch::from_gregorian_utc(2021, 12, 28, 01, 01, 00, 00)
    );

    generic_observation_rinex_test(
            &dut,
            "3.04",
            Some("MIXED"),
            false,
            "G01, G07, G08, G10, G16, G18, G21, G23, G26, G30, R04, R05, R10, R12, R20, R21, E02, E11, E12, E24, E25, E31, E33, E36, C05, C11, C14, C21, C22, C23, C25, C28, C34, C37, C42, C43, C44, C58",
            "GPS, GLO, GAL, BDS",
            &[
                ("GPS", "C1C, L1C, S1C, C2S, L2S, S2S, C2W, L2W, S2W, C5Q, L5Q, S5Q"),
                ("Gal", "C1C, L1C, S1C, C5Q, L5Q, S5Q, C6C, L6C, S6C, C7Q, L7Q, S7Q, C8Q, L8Q, S8Q"),
                ("GLO", "C1C, L1C, S1C, C2P, L2P, S2P, C2C, L2C, S2C, C3Q, L3Q, S3Q"),
                ("BDS", "C2I, L2I, S2I, C6I, L6I, S6I, C7I, L7I, S7I"),
            ],
            Some("2021-12-21T00:00:00 GPST"),
            Some("2021-12-21T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2021-12-21T00:00:00 GPST, 2021-12-21T00:12:00 GPST, 30 s"),
            vec![

            ],
            vec![],
        );
}

#[test]
#[cfg(feature = "flate2")]
fn v3_esbc00dnk() {
    let dut = Rinex::from_gzip_file(
        "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
    )
    .unwrap();

    let mut geo_marker = GeodeticMarker::default()
        .with_name("ESBC00DNK")
        .with_number("10118M001");

    geo_marker.marker_type = Some(MarkerType::Geodetic);

    generic_observation_rinex_test(
            &dut,
            "3.05",
            Some("MIXED"),
            false,
            "C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C16, C19, C20, C21, C22, C23, C24, C25, C26, C27, C28, C29, C30, C32, C33, C34, C35, C36, C37,
                 E01, E02, E03, E04, E05, E07, E08, E09, E11, E12, E13, E15, E19, E21, E24, E25, E26, E27, E30, E31, E33, E36,
                 G01, G02, G03, G04, G05, G06, G07, G08, G09, G10, G11, G12, G13, G14, G15, G16, G17, G18, G19, G20, G21, G22, G24, G25, G26, G27, G28, G29, G30, G31, G32,
                 R01, R02, R03, R04, R05, R06, R07, R08, R09, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R23, R24,
                 J01, J02, J03,
                 S23, S25, S26, S36, S44",
            "BDS, GAL, GLO, QZSS, GPS, EGNOS, SDCM, BDSBAS",
            &[
                ("GPS", "C1C, C1W, C2L, C2W, C5Q, D1C, D2L, D2W, D5Q, L1C, L2L, L2W, L5Q, S1C, S1W, S2L, S2W, S5Q"),
                ("BDS", "C2I, C6I, C7I, D2I, D6I, D7I, L2I, L6I, L7I, S2I, S6I, S7I"),
                ("GAL", "C1C, C5Q, C6C, C7Q, C8Q, D1C, D5Q, D6C, D7Q, D8Q, L1C, L5Q, L6C, L7Q, L8Q, S1C, S5Q, S6C, S7Q, S8Q"),
                ("GLO", "C1C, C1P, C2C, C2P, C3Q, D1C, D1P, D2C, D2P, D3Q, L1C, L1P, L2C, L2P, L3Q, S1C, S1P, S2C, S2P, S3Q"),
                ("SBAS", "C1C, C5I, D1C, D5I, L1C, L5I, S1C, S5I"),
                ("QZSS", "C1C, C2L, C5Q, D1C, D2L, D5Q, L1C, L2L, L5Q, S1C, S2L, S5Q"),
            ],
            Some("2020-06-25T00:00:00 GPST"),
            Some("2020-06-25T23:59:30 GPST"),
            None,
            None,
            Some(geo_marker),
            TimeFrame::from_inclusive_csv(
                "2020-06-25T00:00:00 GPST, 2020-06-25T23:59:30 GPST, 30 s",
            ),
            vec![],
            vec![],
        );
}

#[test]
#[cfg(feature = "flate2")]
fn v3_mojn00dnk() {
    let dut = Rinex::from_gzip_file(
        "../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz",
    )
    .unwrap();

    generic_observation_rinex_test(
            &dut,
            "3.05",
            Some("MIXED"),
            false,
            "
                G01, G02, G03, G04, G05, G06, G07, G08, G09, G10, G11, G12, G13, G14, G15, G16, G17, G18, G19, G20, G21, G22, G24, 
                G25, G26, G27, G28, G29, G30, G31, G32,
                C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C16, C19, C20, C21, C22, C23, C24, C25, C26, C27, C28, C29, C30, 
                C32, C33, C34, C35, C36, C37,
                R01, R02, R03, R04, R05, R06, R07, R08, R09, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R23, R24,
                E01, E02, E03, E04, E05, E07, E08, E09, E11, E12, E13, E15, E19, E21, E24, E25, E26, E27, E30, E31, E33, E36,
                J01, J02, J03,
                I01, I02, I04, I05, I06, I09,
                S23, S25, S26, S27, S36, S44
            ",
            "GPS, GLO, BDS, GAL, QZSS, IRNSS, EGNOS, GAGAN, BDSBAS, SDCM",
            &[
                ("GPS", "C1C, C1W, C2L, C2W, C5Q, D1C, D2L, D2W, D5Q, L1C, L2L, L2W, L5Q, S1C, S1W, S2L, S2W, S5Q"),
                ("BDS", "C2I, C6I, C7I, D2I, D6I, D7I, L2I, L6I, L7I, S2I, S6I, S7I"),
                ("Gal", "C1C, C5Q, C6C, C7Q, C8Q, D1C, D5Q, D6C, D7Q, D8Q, L1C, L5Q, L6C, L7Q, L8Q, S1C, S5Q, S6C, S7Q, S8Q"),
                ("IRNSS", "C5A, D5A, L5A, S5A"),
                ("QZSS", "C1C, C2L, C5Q, D1C, D2L, D5Q, L1C, L2L, L5Q, S1C, S2L, S5Q"),
                ("GLO", "C1C, C1P, C2C, C2P, C3Q, D1C, D1P, D2C, D2P, D3Q, L1C, L1P, L2C, L2P, L3Q, S1C, S1P, S2C, S2P, S3Q"),
                ("SBAS", "C1C, C5I, D1C, D5I, L1C, L5I, S1C, S5I"),
            ],
            Some("2020-06-25T00:00:00 GPST"),
            Some("2020-06-25T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2020-06-25T00:00:00 GPST, 2020-06-25T23:59:30 GPST, 30 s"),
            vec![],
            vec![]
        );
}
