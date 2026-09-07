use crate::{
    navigation::{
        timescale, BdModel, EarthOrientation, Ephemeris, GloCdmaModel, IonosphereModel, KbModel,
        KbRegionCode, NavFrame, NavFrameType, NavKey, NavMessageSubtype, NavMessageType,
        NavicKbModel, NavicNeqnModel, NgModel, SystemTime,
    },
    prelude::{Constellation, ParsingError, SV},
};

/// ([NavKey], [NavFrame]) parsing attempt for a V4 frame.
/// In modern Navigation, all forms may exist.
pub fn parse(content: &str) -> Result<(NavKey, NavFrame), ParsingError> {
    let mut lines = content.lines();

    let line = match lines.next() {
        Some(l) => l,
        _ => return Err(ParsingError::EmptyEpoch),
    };

    let (_, rem) = line.split_at(2);
    let (class, rem) = rem.split_at(4);
    let (svnn, rem) = rem.split_at(4);

    // frmtype defines message to follow
    let frmtype = class.trim().parse::<NavFrameType>()?;
    let sv = svnn.trim().parse::<SV>()?;

    // message type, followed by an optional subtype (RINEX 4.02)
    let mut items = rem.split_ascii_whitespace();

    let msgtype = items
        .next()
        .ok_or(ParsingError::NavMsgType)?
        .parse::<NavMessageType>()?;

    let subtype = match items.next() {
        Some(item) => Some(item.parse::<NavMessageSubtype>()?),
        None => None,
    };

    let ts = timescale(sv.constellation)?;

    // Parses navframe type dependent and epoch of publication
    let (epoch, fr) = match frmtype {
        NavFrameType::Ephemeris => {
            let (epoch, _, ephemeris) = Ephemeris::parse_v4(msgtype, lines, ts)?;
            (epoch, NavFrame::EPH(ephemeris))
        },
        NavFrameType::IonosphereModel => {
            let (epoch, model) = match msgtype {
                NavMessageType::IFNV => {
                    let (epoch, model) = NgModel::parse(lines, ts)?;
                    (epoch, IonosphereModel::NequickG(model))
                },
                // RINEX 4.02: NavIC L1NV models, told apart by the subtype
                NavMessageType::L1NV => match subtype {
                    Some(NavMessageSubtype::KLOB) => {
                        let (epoch, model) = NavicKbModel::parse(lines, ts)?;
                        (epoch, IonosphereModel::NavicKlobuchar(model))
                    },
                    Some(NavMessageSubtype::NEQN) => {
                        let (epoch, model) = NavicNeqnModel::parse(lines, ts)?;
                        (epoch, IonosphereModel::NavicNequick(model))
                    },
                    _ => return Err(ParsingError::NavMsgSubtype),
                },
                // RINEX 4.02: GLONASS CDMA model
                NavMessageType::LXOC => {
                    let (epoch, model) = GloCdmaModel::parse(lines, ts)?;
                    (epoch, IonosphereModel::GlonassCdma(model))
                },
                NavMessageType::CNVX => match sv.constellation {
                    Constellation::BeiDou => {
                        let (epoch, model) = BdModel::parse(lines, ts)?;
                        (epoch, IonosphereModel::Bdgim(model))
                    },
                    _ => {
                        let (epoch, mut model) = KbModel::parse(lines, ts)?;
                        // RINEX 4.02: QZSS region code moved to the subtype field
                        match subtype {
                            Some(NavMessageSubtype::WIDE) => model.region = KbRegionCode::WideArea,
                            Some(NavMessageSubtype::JAPN) => model.region = KbRegionCode::JapanArea,
                            _ => {},
                        }
                        (epoch, IonosphereModel::Klobuchar(model))
                    },
                },
                _ => {
                    let (epoch, model) = KbModel::parse(lines, ts)?;
                    (epoch, IonosphereModel::Klobuchar(model))
                },
            };
            (epoch, NavFrame::ION(model))
        },
        NavFrameType::SystemTimeOffset => {
            // grab next lines
            let line_1 = lines.next().ok_or(ParsingError::EmptyEpoch)?;
            let line_2 = lines.next().ok_or(ParsingError::EmptyEpoch)?;
            let (epoch, system_time) = SystemTime::parse(line_1, line_2, ts)?;
            (epoch, NavFrame::STO(system_time))
        },
        NavFrameType::EarthOrientation => {
            // grab next lines
            let line_1 = lines.next().ok_or(ParsingError::EmptyEpoch)?;
            let line_2 = lines.next().ok_or(ParsingError::EmptyEpoch)?;
            let line_3 = lines.next().ok_or(ParsingError::EmptyEpoch)?;
            let (epoch, eop) = EarthOrientation::parse(line_1, line_2, line_3, ts)?;
            (epoch, NavFrame::EOP(eop))
        },
    };

    let key = NavKey {
        epoch,
        sv,
        msgtype,
        subtype,
        frmtype,
    };

    Ok((key, fr))
}

#[cfg(test)]
mod test {
    use super::parse;
    use crate::{
        navigation::{
            IonosphereModel, IrnssHealth, KbRegionCode, NavFrame, NavFrameType, NavMessageSubtype,
            NavMessageType, OrbitItem,
        },
        prelude::{Epoch, ParsingError, TimeScale, SV},
    };
    use std::str::FromStr;

    #[test]
    fn record_header_without_subtype() {
        // RINEX 4.00 record header: message type only
        let content = "> ION G14 CNVX
    2021 07 05 23 30 42 7.450580596924e-09 2.235174179077e-08-5.960464477539e-08
    -1.192092895508e-07 9.216000000000e+04 1.310720000000e+05-6.553600000000e+04
    -5.242880000000e+05
";
        let (key, frame) = parse(content).unwrap();
        assert_eq!(key.sv, SV::from_str("G14").unwrap());
        assert_eq!(key.frmtype, NavFrameType::IonosphereModel);
        assert_eq!(key.msgtype, NavMessageType::CNVX);
        assert_eq!(key.subtype, None);
        assert_eq!(
            key.epoch,
            Epoch::from_str("2021-07-05T23:30:42 GPST").unwrap()
        );
        match frame {
            NavFrame::ION(IonosphereModel::Klobuchar(model)) => {
                assert_eq!(model.alpha.0, 7.450580596924e-09);
                assert_eq!(model.beta.3, -5.242880000000e+05);
                assert_eq!(model.region, KbRegionCode::WideArea);
            },
            _ => panic!("wrong frame: {:?}", frame),
        }
    }

    #[test]
    fn record_header_with_subtype() {
        // RINEX 4.02 (Table 25): QZSS region code as message subtype
        for (subtype, region) in [
            (NavMessageSubtype::WIDE, KbRegionCode::WideArea),
            (NavMessageSubtype::JAPN, KbRegionCode::JapanArea),
        ] {
            let content = format!(
                "> ION J01 CNVX {}
    2021 07 05 23 30 42 7.450580596924e-09 2.235174179077e-08-5.960464477539e-08
    -1.192092895508e-07 9.216000000000e+04 1.310720000000e+05-6.553600000000e+04
    -5.242880000000e+05
",
                subtype
            );
            let (key, frame) = parse(&content).unwrap();
            assert_eq!(key.sv, SV::from_str("J01").unwrap());
            assert_eq!(key.msgtype, NavMessageType::CNVX);
            assert_eq!(key.subtype, Some(subtype));
            match frame {
                NavFrame::ION(IonosphereModel::Klobuchar(model)) => {
                    assert_eq!(model.region, region);
                },
                _ => panic!("wrong frame: {:?}", frame),
            }
        }
    }

    #[test]
    fn glonass_lxoc_sto() {
        // RINEX 4.02 Table A41
        let content = "> STO R26 LXOC
    2024 02 03 00 15 00 GLUT                                  UTC(SU)
     5.184000000000e+05-8.335337042809e-08-4.618527782441e-14 0.000000000000e+00
";
        let (key, frame) = parse(content).unwrap();
        assert_eq!(key.sv, SV::from_str("R26").unwrap());
        assert_eq!(key.frmtype, NavFrameType::SystemTimeOffset);
        assert_eq!(key.msgtype, NavMessageType::LXOC);
        assert_eq!(key.subtype, None);
        assert_eq!(
            key.epoch,
            Epoch::from_str("2024-02-03T00:15:00 UTC").unwrap()
        );
        match frame {
            NavFrame::STO(sto) => {
                assert_eq!(sto.system, "GLUT");
                assert_eq!(sto.utc, "UTC(SU)");
                assert_eq!(sto.t_tm, 518400);
                assert_eq!(sto.a, (-8.335337042809e-08, -4.618527782441e-14, 0.0));
            },
            _ => panic!("wrong frame: {:?}", frame),
        }
    }

    #[test]
    fn glonass_lxoc_eop() {
        // RINEX 4.02 Table A41
        let content = "> EOP R04 LXOC
    2024 02 02 21 00 00 6.079101562500e-02-2.380371093750e-03 0.000000000000e+00
                        2.203369140625e-01 1.281738281250e-03 0.000000000000e+00
     5.183940000000e+05 3.387451171875e-03-2.441406250000e-04 0.000000000000e+00
";
        let (key, frame) = parse(content).unwrap();
        assert_eq!(key.sv, SV::from_str("R04").unwrap());
        assert_eq!(key.frmtype, NavFrameType::EarthOrientation);
        assert_eq!(key.msgtype, NavMessageType::LXOC);
        assert_eq!(
            key.epoch,
            Epoch::from_str("2024-02-02T21:00:00 UTC").unwrap()
        );
        match frame {
            NavFrame::EOP(eop) => {
                assert_eq!(eop.x.0, 6.079101562500e-02);
                assert_eq!(eop.y.0, 2.203369140625e-01);
                assert_eq!(eop.t_tm, 518394);
            },
            _ => panic!("wrong frame: {:?}", frame),
        }
    }

    #[test]
    fn navic_lnav_ephemeris() {
        // RINEX 4.02 Table A32
        let content = "> EPH I02 LNAV
I02 2020 09 15 02 05 36 6.225099787116e-04 1.773514668457e-11 0.000000000000e+00
     1.690000000000e+02-5.793750000000e+02 4.834487090078e-09-4.281979621524e-01
    -1.904368400574e-05 2.015684265643e-03-3.430992364883e-06 6.493289550781e+03
     1.803360000000e+05 2.495944499969e-07-1.337499015334e+00 7.450580596924e-08
     5.022043764738e-01 1.946250000000e+02-2.970970345572e+00-4.461614415577e-09
    -9.578970431139e-10                    2.123000000000e+03
     2.000000000000e+00 0.000000000000e+00-1.862645149231e-09
     1.804920000000e+05
";
        let (key, frame) = parse(content).unwrap();
        assert_eq!(key.sv, SV::from_str("I02").unwrap());
        assert_eq!(key.frmtype, NavFrameType::Ephemeris);
        assert_eq!(key.msgtype, NavMessageType::LNAV);
        // NavIC time is expressed in GPST
        assert_eq!(
            key.epoch,
            Epoch::from_str("2020-09-15T02:05:36 GPST").unwrap()
        );
        let eph = frame.as_ephemeris().unwrap();
        assert_eq!(eph.clock_bias, 6.225099787116e-04);
        assert_eq!(eph.clock_drift, 1.773514668457e-11);
        assert_eq!(eph.get_orbit_f64("iodec"), Some(1.690000000000e+02));
        assert_eq!(eph.get_orbit_f64("crs"), Some(-5.793750000000e+02));
        assert_eq!(eph.get_orbit_f64("sqrta"), Some(6.493289550781e+03));
        assert_eq!(eph.get_orbit_f64("toe"), Some(1.803360000000e+05));
        assert_eq!(eph.get_orbit_f64("idot"), Some(-9.578970431139e-10));
        assert_eq!(eph.get_week(), Some(2123));
        assert_eq!(eph.get_orbit_f64("ura"), Some(2.0));
        assert_eq!(
            eph.orbits.get("health"),
            Some(&OrbitItem::IrnssHealth(IrnssHealth::Healthy))
        );
        assert_eq!(eph.get_orbit_f64("tgd"), Some(-1.862645149231e-09));
        assert_eq!(eph.get_orbit_f64("t_tm"), Some(1.804920000000e+05));
        assert_eq!(
            eph.toe(TimeScale::GPST),
            Some(Epoch::from_str("2020-09-15T02:05:36 GPST").unwrap())
        );
    }

    #[test]
    fn navic_l1nv_ephemeris() {
        // RINEX 4.02 Table A32
        let content = "> EPH I10 L1NV
I10 2023 06 24 00 05 00 1.527369022369e-07 1.364242052659e-12 0.000000000000e+00
     0.000000000000e+00-2.593125000000e+02 7.028864208979e-09 2.300305834983e+00
    -8.691102266312e-06 4.531537415460e-04-3.855675458908e-06 6.493495117188e+03
     7.000000000000e+00 1.341104507446e-07 1.359342162629e-01-7.078051567078e-08
     8.594830530333e-02 1.214375000000e+02-5.136403830694e-02-5.858815471753e-09
    -4.846630453041e-10 0.000000000000e+00                    1.000000000000e+00
     1.500000000000e+01 0.000000000000e+00                   -3.608874976635e-09
                                           6.984919309616e-09 5.995389074087e-09
     5.191380000000e+05
";
        let (key, frame) = parse(content).unwrap();
        assert_eq!(key.sv, SV::from_str("I10").unwrap());
        assert_eq!(key.msgtype, NavMessageType::L1NV);
        assert_eq!(
            key.epoch,
            Epoch::from_str("2023-06-24T00:05:00 GPST").unwrap()
        );
        let eph = frame.as_ephemeris().unwrap();
        assert_eq!(eph.clock_bias, 1.527369022369e-07);
        assert_eq!(eph.get_orbit_f64("crs"), Some(-2.593125000000e+02));
        assert_eq!(eph.get_orbit_f64("deltaN0"), Some(7.028864208979e-09));
        assert_eq!(eph.get_orbit_f64("iodec"), Some(7.0));
        assert_eq!(eph.get_orbit_f64("cic"), Some(1.341104507446e-07));
        assert_eq!(eph.get_orbit_f64("omegaDot"), Some(-5.858815471753e-09));
        assert_eq!(eph.get_orbit_f64("idot"), Some(-4.846630453041e-10));
        assert_eq!(eph.get_orbit_f64("rsf"), Some(1.0));
        assert_eq!(eph.get_orbit_f64("urai"), Some(15.0));
        assert_eq!(
            eph.orbits.get("health"),
            Some(&OrbitItem::IrnssHealth(IrnssHealth::Healthy))
        );
        // RSF=1: only the S-L5 group delay and the ISC(S) terms are given
        assert_eq!(eph.get_orbit_f64("tgdL1PL5"), None);
        assert_eq!(eph.get_orbit_f64("tgdSL5"), Some(-3.608874976635e-09));
        assert_eq!(eph.get_orbit_f64("iscSL1P"), None);
        assert_eq!(eph.get_orbit_f64("iscL1DL1P"), None);
        assert_eq!(eph.get_orbit_f64("iscL1PS"), Some(6.984919309616e-09));
        assert_eq!(eph.get_orbit_f64("iscL1DS"), Some(5.995389074087e-09));
        assert_eq!(eph.get_orbit_f64("t_tm"), Some(5.191380000000e+05));
    }

    #[test]
    fn glonass_cdma_ephemeris() {
        // RINEX 4.02 Table A18: L1OC and L3OC share the same layout,
        // except for the group delay field on line 3.
        for (msgtype, delay) in [
            (NavMessageType::L1OC, "tgdL2OCp"),
            (NavMessageType::L3OC, "iscL3OCp"),
        ] {
            let content = format!(
                "> EPH R26 {}
R26 2024 02 03 00 15 00-1.605716170161e-05 1.652011860642e-12-2.081668171172e-17
     1.812154053020e+04-2.071979139000e+00 5.729816621169e-10 1.000000000000e+00
    -2.325615360260e+03 1.285475494340e+00-1.047737896442e-09 1.000000000000e+00
     1.781341854668e+04 2.280306640081e+00-9.458744898438e-10 0.000000000000e+00
     2.000000000000e+00 1.000000000000e+01 7.500000000000e-01 7.500000000000e-01
     0.000000000000e+00 0.000000000000e+00 0.000000000000e+00 0.000000000000e+00
     0.000000000000e+00 0.000000000000e+00 0.000000000000e+00 0.000000000000e+00
     0.000000000000e+00 0.000000000000e+00 0.000000000000e+00 0.000000000000e+00
     1.500000000000e+01 5.000000000000e+00                    5.184000000000e+05
",
                msgtype
            );
            let (key, frame) = parse(&content).unwrap();
            assert_eq!(key.sv, SV::from_str("R26").unwrap());
            assert_eq!(key.frmtype, NavFrameType::Ephemeris);
            assert_eq!(key.msgtype, msgtype);
            assert_eq!(key.subtype, None);
            assert_eq!(
                key.epoch,
                Epoch::from_str("2024-02-03T00:15:00 UTC").unwrap()
            );
            let eph = frame.as_ephemeris().unwrap();
            assert_eq!(eph.clock_bias, -1.605716170161e-05);
            assert_eq!(eph.clock_drift, 1.652011860642e-12);
            assert_eq!(eph.clock_drift_rate, -2.081668171172e-17);
            assert_eq!(eph.get_orbit_f64("satPosX"), Some(1.812154053020e+04));
            assert_eq!(eph.get_orbit_f64("velX"), Some(-2.071979139000e+00));
            assert_eq!(eph.get_orbit_f64("accelX"), Some(5.729816621169e-10));
            assert!(eph.orbits.get("health").is_some());
            assert_eq!(eph.get_orbit_f64("satPosY"), Some(-2.325615360260e+03));
            assert_eq!(
                eph.orbits.get("dataValidity").and_then(|v| v.as_u8()),
                Some(1)
            );
            assert_eq!(eph.get_orbit_f64("satPosZ"), Some(1.781341854668e+04));
            assert_eq!(eph.get_orbit_f64("accelZ"), Some(-9.458744898438e-10));
            // zero valued: get_orbit_f64 hides zeros
            assert_eq!(eph.orbits.get(delay).and_then(|v| v.as_f64()), Some(0.0));
            assert_eq!(eph.orbits.get("satType").and_then(|v| v.as_u8()), Some(2));
            assert_eq!(
                eph.orbits.get("sourceFlags").and_then(|v| v.as_u8()),
                Some(10)
            );
            assert_eq!(eph.get_orbit_f64("aode"), Some(0.75));
            assert_eq!(eph.get_orbit_f64("aodc"), Some(0.75));
            assert_eq!(eph.get_orbit_f64("uraiOrb"), Some(15.0));
            assert_eq!(eph.get_orbit_f64("uraiClk"), Some(5.0));
            assert_eq!(eph.get_orbit_f64("t_tm"), Some(5.184000000000e+05));
            // no frequency channel in CDMA messages
            assert_eq!(eph.glonass_freq_channel(), None);
        }
    }

    #[test]
    fn navic_l1nv_ionosphere() {
        // RINEX 4.02 Table A41: same message type, told apart by the subtype
        let klob = "> ION I10 L1NV KLOB
    2023 06 24 00 07 30 1.000000000000e+00
     5.867332220078e-08 2.533197402954e-07-1.430511474609e-06-7.510185241699e-06
     1.495040000000e+05-5.406720000000e+05 2.883584000000e+06 8.323072000000e+06
     5.000000000000e+01 1.100000000000e+02 0.000000000000e+00 5.000000000000e+01
";
        let (key, frame) = parse(klob).unwrap();
        assert_eq!(key.sv, SV::from_str("I10").unwrap());
        assert_eq!(key.frmtype, NavFrameType::IonosphereModel);
        assert_eq!(key.msgtype, NavMessageType::L1NV);
        assert_eq!(key.subtype, Some(NavMessageSubtype::KLOB));
        assert_eq!(
            key.epoch,
            Epoch::from_str("2023-06-24T00:07:30 GPST").unwrap()
        );
        let model = frame
            .as_ionosphere_model()
            .and_then(|m| m.as_navic_klobuchar())
            .unwrap();
        assert_eq!(model.iodk, 1.0);
        assert_eq!(model.alpha.0, 5.867332220078e-08);
        assert_eq!(model.beta.3, 8.323072000000e+06);
        assert_eq!(model.longitude_deg, (50.0, 110.0));
        assert_eq!(model.latitude_deg, (0.0, 50.0));

        let neqn = "> ION I10 L1NV NEQN
    2023 06 24 00 17 24 0.000000000000e+00
     1.982500000000e+02 0.000000000000e+00 0.000000000000e+00 1.000000000000e+00
     3.000000000000e+01 1.300000000000e+02-3.000000000000e+01-1.000000000000e+01
     2.085000000000e+02-3.085937500000e-01-3.417968750000e-03 1.000000000000e+00
     3.000000000000e+01 1.300000000000e+02-5.000000000000e+00 3.000000000000e+01
     1.982500000000e+02 0.000000000000e+00 0.000000000000e+00 1.000000000000e+00
     3.000000000000e+01 1.300000000000e+02 3.500000000000e+01 5.000000000000e+01
";
        let (key, frame) = parse(neqn).unwrap();
        assert_eq!(key.msgtype, NavMessageType::L1NV);
        assert_eq!(key.subtype, Some(NavMessageSubtype::NEQN));
        assert_eq!(
            key.epoch,
            Epoch::from_str("2023-06-24T00:17:24 GPST").unwrap()
        );
        let model = frame
            .as_ionosphere_model()
            .and_then(|m| m.as_navic_nequick())
            .unwrap();
        assert_eq!(model.iodn, 0.0);
        assert_eq!(model.regions[1].a.0, 208.5);
        assert_eq!(model.regions[2].modip_deg, (35.0, 50.0));

        // the subtype is mandatory for L1NV ION records
        let missing = klob.replace(" KLOB", "");
        assert!(matches!(parse(&missing), Err(ParsingError::NavMsgSubtype)));
    }

    #[test]
    fn glonass_lxoc_ionosphere() {
        // RINEX 4.02 Table A41
        let content = "> ION R22 LXOC
    2024 02 03 00 01 09 1.000000000000e+00 1.410000000000e+02 5.000000000000e+00
";
        let (key, frame) = parse(content).unwrap();
        assert_eq!(key.sv, SV::from_str("R22").unwrap());
        assert_eq!(key.frmtype, NavFrameType::IonosphereModel);
        assert_eq!(key.msgtype, NavMessageType::LXOC);
        assert_eq!(key.subtype, None);
        assert_eq!(
            key.epoch,
            Epoch::from_str("2024-02-03T00:01:09 UTC").unwrap()
        );
        let model = frame
            .as_ionosphere_model()
            .and_then(|m| m.as_glonass_cdma())
            .unwrap();
        assert_eq!((model.c_a, model.c_f107, model.c_ap), (1.0, 141.0, 5.0));
    }

    #[test]
    fn gps_cnav_flags() {
        // RINEX 4.02 Table A10: optional integer flags after wn_op
        // (bit 0 integrity status, bit 1 L2C phasing, bit 2 alert)
        let content = "> EPH G04 CNAV
G04 2019 03 14 03 30 00 1.330042141490e-04 7.226219622680e-12 0.000000000000e+00
     2.001762390137e-03 6.914062500000e-01 4.625906973308e-09 1.887277537485e+00
     1.024454832077e-08 3.348654136062e-04 8.376315236092e-06 5.153800325291e+03
     2.412000000000e+05-4.656612873077e-09 5.171544951605e-01 2.328306436539e-08
     9.601927657114e-01 2.174140625000e+02-1.737767543851e+00-8.034028170143e-09
    -2.950122884460e-10-1.312310522376e-14-2.000000000000e+00 2.000000000000e+00
     0.000000000000e+00 7.000000000000e+00-8.789356797934e-09 5.000000000000e+00
    -5.820766091347e-10-6.606569513679e-09-1.178705133498e-08-1.178705133498e-08
     3.558540000000e+05 2.044000000000e+03";

        // RINEX 4.00 / 4.01: no flags (Table A12 example)
        let (key, frame) = parse(content).unwrap();
        assert_eq!(key.msgtype, NavMessageType::CNAV);
        let eph = frame.as_ephemeris().unwrap();
        assert_eq!(eph.get_orbit_f64("t_tm"), Some(3.558540000000e+05));
        assert_eq!(eph.get_orbit_f64("wn_op"), Some(2044.0));
        assert_eq!(eph.orbits.get("flags"), None);

        // RINEX 4.02: flags = 5 (integrity status and alert)
        let content = format!("{} 5.000000000000e+00\n", content);
        let (_, frame) = parse(&content).unwrap();
        let eph = frame.as_ephemeris().unwrap();
        assert_eq!(eph.get_orbit_f64("t_tm"), Some(3.558540000000e+05));
        assert_eq!(eph.get_orbit_f64("wn_op"), Some(2044.0));
        assert_eq!(eph.orbits.get("flags").and_then(|v| v.as_u8()), Some(5));
    }

    #[test]
    fn record_header_bad_subtype() {
        let content = "> ION J01 CNVX XXXX
    2021 07 05 23 30 42 7.450580596924e-09 2.235174179077e-08-5.960464477539e-08
    -1.192092895508e-07 9.216000000000e+04 1.310720000000e+05-6.553600000000e+04
    -5.242880000000e+05
";
        assert!(matches!(parse(content), Err(ParsingError::NavMsgSubtype)));
    }
}
