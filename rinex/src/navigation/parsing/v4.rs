use crate::{
    navigation::{
        BdModel, EarthOrientation, Ephemeris, IonosphereModel, KbModel, KbRegionCode, NavFrame,
        NavFrameType, NavKey, NavMessageSubtype, NavMessageType, NgModel, SystemTime,
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

    let ts = sv
        .constellation
        .timescale()
        .ok_or(ParsingError::NoTimescaleDefinition)?;

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
            IonosphereModel, KbRegionCode, NavFrame, NavFrameType, NavMessageSubtype,
            NavMessageType,
        },
        prelude::{Epoch, ParsingError, SV},
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
    fn record_header_bad_subtype() {
        let content = "> ION J01 CNVX XXXX
    2021 07 05 23 30 42 7.450580596924e-09 2.235174179077e-08-5.960464477539e-08
    -1.192092895508e-07 9.216000000000e+04 1.310720000000e+05-6.553600000000e+04
    -5.242880000000e+05
";
        assert!(matches!(parse(content), Err(ParsingError::NavMsgSubtype)));
    }
}
