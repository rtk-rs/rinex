//! RNX2CRX round trips: every model RINEX is compressed, the CRINEX is
//! written to a file, parsed back and compared point by point with the
//! model.
#[cfg(test)]
mod test {
    use crate::{
        prelude::Rinex,
        tests::toolkit::{generic_observation_comparison, random_name},
    };

    use std::{fs::remove_file as fs_remove_file, path::Path};

    fn run_round_trip_test(rnx_path: &str, crinex_major: u8) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join(rnx_path);

        let model = Rinex::from_file(path.to_string_lossy().as_ref())
            .unwrap_or_else(|e| panic!("failed to parse {}: {}", rnx_path, e));

        // RINEX -> CRINEX
        let compressed = model.rnx2crnx();

        let crinex = compressed
            .header
            .obs
            .as_ref()
            .and_then(|obs| obs.crinex.as_ref())
            .expect("rnx2crnx did not set up a CRINEX header");

        assert_eq!(crinex.version.major, crinex_major);

        let tmp_path = std::env::temp_dir().join(format!("rinex-{}.crx", random_name(8)));
        let tmp_path = tmp_path.to_string_lossy().to_string();

        compressed
            .to_file(&tmp_path)
            .unwrap_or_else(|e| panic!("failed to write {}: {}", tmp_path, e));

        // CRINEX -> RINEX
        let dut = Rinex::from_file(&tmp_path)
            .unwrap_or_else(|e| panic!("failed to parse compressed {}: {}", rnx_path, e));

        let _ = fs_remove_file(&tmp_path);

        assert!(
            dut.header.obs.as_ref().unwrap().crinex.is_some(),
            "compressed file was not recognized as CRINEX"
        );

        assert_eq!(
            dut.epoch_iter().count(),
            model.epoch_iter().count(),
            "{}: wrong number of epochs after round trip",
            rnx_path
        );

        // strict point by point comparison of every signal and clock offset
        generic_observation_comparison(&dut, &model);
    }

    #[test]
    fn crinex1_round_trip() {
        for rnx_name in [
            "AJAC3550.21O",
            "aopr0010.17o",
            "npaz3550.21o",
            "wsra0010.21o",
            "zegv0010.21o",
            "delf0010_clock.21o",
        ] {
            run_round_trip_test(&format!("OBS/V2/{}", rnx_name), 1);
        }
    }

    #[test]
    fn crinex3_round_trip() {
        for rnx_name in [
            "ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            "DUTH0630.22O",
            "VLNS0010.22O",
            "VLNS0630.22O",
            "pdel0010.21o",
            "ACRG00GHA_R_20240010000_01H_30S_MO.rnx",
        ] {
            run_round_trip_test(&format!("OBS/V3/{}", rnx_name), 3);
        }
    }

    /// RINEX 4 observation files are compressed as CRINEX 3
    #[test]
    fn rinex4_round_trip() {
        run_round_trip_test("OBS/V4/ACRG00GHA_R_20240010000_01H_30S_MO.rnx", 3);
    }
}
