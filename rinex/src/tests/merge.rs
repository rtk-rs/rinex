#[cfg(test)]
mod test {
    use crate::{
        prelude::{qc::Merge, Rinex},
        tests::toolkit::{generic_observation_rinex_test, TimeFrame},
    };
    use std::{fs::remove_file as fs_remove_file, path::PathBuf};

    #[test]
    fn fail_on_type_mismatch() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");

        let path1 = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("AMEL00NLD_R_20210010000_01D_MN.rnx");

        let path2 = test_resources
            .clone()
            .join("OBS")
            .join("V3")
            .join("LARM0630.22O");

        let path = path1.to_string_lossy().to_string();
        let mut r1 = Rinex::from_file(&path).unwrap();

        let path = path2.to_string_lossy().to_string();
        let r2 = Rinex::from_file(&path).unwrap();

        assert!(r1.merge_mut(&r2).is_err())
    }

    #[test]
    fn merge_nav() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");

        let path = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("AMEL00NLD_R_20210010000_01D_MN.rnx");

        let path = path.to_string_lossy().to_string();
        let rnx_a = Rinex::from_file(&path).unwrap();

        let path = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("CBW100NLD_R_20210010000_01D_MN.rnx");

        let path = path.to_string_lossy().to_string();
        let rnx_b = Rinex::from_file(&path).unwrap();

        let merged = rnx_a.merge(&rnx_b);
        assert!(
            merged.is_ok(),
            "failed to merge NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx into NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx"
        );

        let merged = merged.unwrap();

        merged
            .to_file("cbw-merged.txt")
            .expect("failed to generate file!");

        assert!(merged.is_merged());

        // parse back
        let rnx = Rinex::from_file("cbw-merged.txt").unwrap();

        assert!(rnx.is_merged());

        /*
         * Unlock reciprocity test in near future
         *  NAV file production does not work correctly at the moment,
         *  due to formatting issues
         */
        // assert_eq!(rnx, merged, "Merge::ops reciprocity");

        // remove file we just generated
        let _ = std::fs::remove_file("cbw-merged.txt");
    }

    #[test]
    fn merge_obs() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");

        let path = test_resources
            .clone()
            .join("OBS")
            .join("V2")
            .join("AJAC3550.21O");

        let path = path.to_string_lossy().to_string();
        let rnx_a = Rinex::from_file(&path).unwrap();

        let path = test_resources
            .clone()
            .join("OBS")
            .join("V2")
            .join("npaz3550.21o");

        let path = path.to_string_lossy().to_string();
        let rnx_b = Rinex::from_file(&path).unwrap();

        let merged = rnx_a.merge(&rnx_b);
        let merged = merged.unwrap();
        assert!(merged.is_merged());

        generic_observation_rinex_test(
            &merged,
            "2.11",
            Some("MIXED"),
            false,
            "G01, G07, G08, G10, G15, G16, G18, G21, G23, G26, G32, R04, R05, R06, R07, R10, R12, R19, R20, R21, R22, E04, E11, E12, E19, E24, E25, E31, E33, S23, S36", 
            "GPS, GLO, GAL, EGNOS",
            &[
                ("GPS", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                ("GLO", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                ("GAL", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                ("SBAS", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
            ],
            Some("2021-12-21T00:00:00 GPST"),
            Some("2021-12-21T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2021-12-21T00:00:00 GPST, 2021-12-21T01:04:00 GPST, 30 s"),
            vec![],
            vec![],
        );

        merged.to_file("ajac-merge.txt").unwrap();

        // parse back
        let rnx = Rinex::from_file("ajac-merge.txt").unwrap();

        assert!(
            rnx.is_merged(),
            "failed to identify a merged file correctly"
        );

        // remove file we just generated
        let _ = fs_remove_file("ajac-merged.txt");
    }

    // #[cfg(feature = "antex")]
    // use crate::antex::antenna::AntennaMatcher;
    // #[cfg(feature = "antex")]
    // use crate::Carrier;

    // TODO
    // #[test]
    // #[cfg(feature = "flate2")]
    // #[cfg(feature = "antex")]
    // fn merge_atx() {
    //     let fp = env!("CARGO_MANIFEST_DIR").to_owned()
    //         + "/../test_resources/ATX/V1/TROSAR25.R4__LEIT_2020_09_23.atx";
    //     let rinex_a = Rinex::from_file(&fp);
    //     let rinex_a = rinex_a.unwrap();

    //     let fp =
    //         env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/ATX/V1/igs14_small.atx.gz";
    //     let rinex_b = Rinex::from_file(&fp);
    //     let rinex_b = rinex_b.unwrap();

    //     let merged = rinex_a.merge(&rinex_b);
    //     assert!(merged.is_ok(), "merged atx(a,b) failed");
    //     let merged = merged.unwrap();

    //     let antennas: Vec<_> = merged.antennas().collect();
    //     assert_eq!(antennas.len(), 7, "bad number of antennas");

    //     for (name, expected_apc) in [
    //         ("JPSLEGANT_E", (1.36, -0.43, 35.44)),
    //         ("JPSODYSSEY_I", (1.06, -2.43, 70.34)),
    //         ("TROSAR25.R4", (-0.22, -0.01, 154.88)),
    //     ] {
    //         let fakenow = Epoch::from_gregorian_utc_at_midnight(2023, 01, 01);
    //         let apc = merged.rx_antenna_apc_offset(
    //             fakenow,
    //             AntennaMatcher::IGSCode(name.to_string()),
    //             Carrier::L1,
    //         );
    //         assert!(apc.is_some(), "APC should still be contained after merge()");
    //         assert_eq!(apc.unwrap(), expected_apc);
    //     }
    // }
}
