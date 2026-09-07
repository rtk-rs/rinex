//! Navigation RINEX formatting
use std::io::{BufWriter, Write};

use crate::{
    epoch::format as epoch_format,
    error::FormattingError,
    navigation::{
        orbits::closest_nav_standards, BdModel, EarthOrientation, Ephemeris, GloCdmaModel,
        IonosphereModel, KbModel, KbRegionCode, NavFrameType, NavKey, NavMessageType, NavicKbModel,
        NavicNeqnModel, NgModel, OrbitItem, Record, SystemTime,
    },
    prelude::{Constellation, Header, RinexType, Version},
};

/// Prefix of the lines following the first line of a record:
/// 3 blanks in RINEX 2, 4 blanks in RINEX 3 and 4.
fn line_prefix(major: u8) -> &'static str {
    if major < 3 {
        "   "
    } else {
        "    "
    }
}

/// Formats `value` as E19.12: 12 decimals, two exponent digits,
/// right aligned on 19 characters. `exponent` is 'D' in RINEX 2
/// and 'E' from RINEX 3 on.
fn format_e19(value: f64, exponent: char) -> String {
    let formatted = format!("{:.12E}", value);
    let (mantissa, exp) = formatted
        .split_once('E')
        .expect("{:E} always formats an exponent");
    let exp = exp
        .parse::<i32>()
        .expect("{:E} always formats an integer exponent");
    let sign = if exp < 0 { '-' } else { '+' };
    format!(
        "{:>19}",
        format!("{}{}{}{:02}", mantissa, exponent, sign, exp.abs())
    )
}

/// Numerical value of an [OrbitItem], as stored in the file
fn orbit_value(item: &OrbitItem) -> f64 {
    match item {
        OrbitItem::U8(v) => *v as f64,
        OrbitItem::I8(v) => *v as f64,
        OrbitItem::U32(v) => *v as f64,
        OrbitItem::F64(v) => *v,
        OrbitItem::Health(h) => h.clone() as u8 as f64,
        OrbitItem::GloHealth(h) => h.clone() as u8 as f64,
        OrbitItem::GeoHealth(h) => h.clone() as u8 as f64,
        OrbitItem::IrnssHealth(h) => h.clone() as u8 as f64,
        OrbitItem::GloStatus(s) => s.bits() as f64,
        OrbitItem::GalHealth(h) => h.bits() as f64,
    }
}

/// Formats the record header line (RINEX 4) and the beginning of the
/// first record line: satellite and epoch for an ephemeris, blank
/// satellite field and epoch for the other frames. No line ending.
fn format_epoch_v4<W: Write>(w: &mut BufWriter<W>, k: &NavKey) -> std::io::Result<()> {
    // record header line: "> TYP SVN MSGT [SUBT]"
    if k.sv.prn == 0 {
        // constellation only, blank PRN
        write!(
            w,
            "> {} {:x}   {}",
            k.frmtype, k.sv.constellation, k.msgtype
        )?;
    } else {
        write!(w, "> {} {:x} {}", k.frmtype, k.sv, k.msgtype)?;
    }
    if let Some(subtype) = k.subtype {
        write!(w, " {}", subtype)?;
    }
    writeln!(w)?;

    let epoch = epoch_format(k.epoch, RinexType::NavigationData, 4);
    match k.frmtype {
        NavFrameType::Ephemeris => write!(w, "{:x} {}", k.sv, epoch),
        _ => write!(w, "    {}", epoch),
    }
}

/// Formats the satellite and epoch opening an ephemeris record
/// in RINEX 2 (two digit PRN, two digit year) or RINEX 3.
/// No line ending.
fn format_epoch_v2v3<W: Write>(w: &mut BufWriter<W>, k: &NavKey, major: u8) -> std::io::Result<()> {
    let epoch = epoch_format(k.epoch, RinexType::NavigationData, major);
    if major < 3 {
        write!(w, "{:2} {}", k.sv.prn, epoch)
    } else {
        write!(w, "{:x} {}", k.sv, epoch)
    }
}

/// Orbit fields named differently by the RINEX 3 and RINEX 4
/// entries of the orbit database (BeiDou group delays).
fn orbit_alias(key: &str) -> Option<&'static str> {
    match key {
        "tgdb1b3" => Some("tgd1b1b3"),
        "tgd1b1b3" => Some("tgdb1b3"),
        "tgdb2b3" => Some("tgd2b2b3"),
        "tgd2b2b3" => Some("tgdb2b3"),
        _ => None,
    }
}

/// Message types with a representation prior RINEX 4:
/// one legacy message per constellation.
fn is_legacy_message(msgtype: NavMessageType, constellation: Constellation) -> bool {
    match msgtype {
        NavMessageType::LNAV => true,
        NavMessageType::FDMA => constellation == Constellation::Glonass,
        NavMessageType::INAV | NavMessageType::FNAV => constellation == Constellation::Galileo,
        NavMessageType::D1 | NavMessageType::D2 => constellation == Constellation::BeiDou,
        NavMessageType::SBAS => constellation.is_sbas(),
        _ => false,
    }
}

/// RINEX 4 message type of an ephemeris parsed from RINEX 2 or 3,
/// where every ephemeris is filed as LNAV.
fn v4_message_type(k: &NavKey, eph: &Ephemeris) -> NavMessageType {
    if k.msgtype != NavMessageType::LNAV {
        return k.msgtype;
    }
    match k.sv.constellation {
        Constellation::Glonass => NavMessageType::FDMA,
        Constellation::Galileo => {
            // Table A13: data source bit 1 set for F/NAV E5a-I
            let data_source = eph.get_orbit_f64("dataSrc").unwrap_or(0.0) as u32;
            if data_source & 0x02 != 0 {
                NavMessageType::FNAV
            } else {
                NavMessageType::INAV
            }
        },
        Constellation::BeiDou => {
            // D2 is broadcast by the GEO satellites (BDS ICD)
            if k.sv.prn <= 5 || k.sv.prn >= 59 {
                NavMessageType::D2
            } else {
                NavMessageType::D1
            }
        },
        c if c.is_sbas() => NavMessageType::SBAS,
        _ => NavMessageType::LNAV,
    }
}

/// Formats the clock fields ending the first line of an ephemeris
/// record, then the orbit lines as described by the orbit database
/// for this revision and message type.
fn format_ephemeris<W: Write>(
    w: &mut BufWriter<W>,
    k: &NavKey,
    msgtype: NavMessageType,
    eph: &Ephemeris,
    version: Version,
    exponent: char,
) -> Result<(), FormattingError> {
    let mut clock_drift_rate = eph.clock_drift_rate;

    // SBAS frames specificity: the drift rate slot carries the week counter
    if k.sv.constellation.is_sbas() {
        if let Some(week) = eph.orbits.get("week").and_then(|v| v.as_u32()) {
            clock_drift_rate = week as f64;
        }
    }

    writeln!(
        w,
        "{}{}{}",
        format_e19(eph.clock_bias, exponent),
        format_e19(eph.clock_drift, exponent),
        format_e19(clock_drift_rate, exponent),
    )?;

    // same normalization as the parser
    let constellation = if k.sv.constellation.is_sbas() {
        Constellation::SBAS
    } else {
        k.sv.constellation
    };

    let standards = closest_nav_standards(constellation, version, msgtype)
        .ok_or(FormattingError::NoNavigationDefinition)?;

    let prefix = line_prefix(version.major);
    let nb_fields = standards.items.len();

    for (index, (key, _)) in standards.items.iter().enumerate() {
        if index % 4 == 0 {
            write!(w, "{}", prefix)?;
        }
        let item = eph
            .orbits
            .get(*key)
            .or_else(|| orbit_alias(key).and_then(|alias| eph.orbits.get(alias)));
        match item {
            Some(item) if !key.contains("spare") => {
                write!(w, "{}", format_e19(orbit_value(item), exponent))?;
            },
            _ => write!(w, "{:19}", "")?,
        }
        if index % 4 == 3 || index == nb_fields - 1 {
            writeln!(w)?;
        }
    }

    Ok(())
}

/// Formats the system time offset record (RINEX 4 Table A33),
/// after the epoch written by [format_epoch_v4].
fn format_system_time<W: Write>(
    w: &mut BufWriter<W>,
    sto: &SystemTime,
    exponent: char,
) -> std::io::Result<()> {
    // time offset codes, SBAS identifier (not stored), UTC identifier
    writeln!(w, " {:<18} {:<18} {:<18}", sto.system, "", sto.utc)?;
    writeln!(
        w,
        "    {}{}{}{}",
        format_e19(sto.t_tm as f64, exponent),
        format_e19(sto.a.0, exponent),
        format_e19(sto.a.1, exponent),
        format_e19(sto.a.2, exponent),
    )
}

/// Formats the Earth orientation record (RINEX 4 Table A34),
/// after the epoch written by [format_epoch_v4].
fn format_earth_orientation<W: Write>(
    w: &mut BufWriter<W>,
    eop: &EarthOrientation,
    exponent: char,
) -> std::io::Result<()> {
    writeln!(
        w,
        "{}{}{}",
        format_e19(eop.x.0, exponent),
        format_e19(eop.x.1, exponent),
        format_e19(eop.x.2, exponent),
    )?;
    writeln!(
        w,
        "    {:19}{}{}{}",
        "",
        format_e19(eop.y.0, exponent),
        format_e19(eop.y.1, exponent),
        format_e19(eop.y.2, exponent),
    )?;
    writeln!(
        w,
        "    {}{}{}{}",
        format_e19(eop.t_tm as f64, exponent),
        format_e19(eop.delta_ut1.0, exponent),
        format_e19(eop.delta_ut1.1, exponent),
        format_e19(eop.delta_ut1.2, exponent),
    )
}

/// Formats a line of up to four E19.12 fields with the 4 blanks prefix
fn format_fields<W: Write>(
    w: &mut BufWriter<W>,
    values: &[f64],
    exponent: char,
) -> std::io::Result<()> {
    write!(w, "    ")?;
    for value in values {
        write!(w, "{}", format_e19(*value, exponent))?;
    }
    writeln!(w)
}

/// Formats the ionosphere model record (RINEX 4 Tables A35 to A40),
/// after the epoch written by [format_epoch_v4].
fn format_ionosphere_model<W: Write>(
    w: &mut BufWriter<W>,
    k: &NavKey,
    model: &IonosphereModel,
    exponent: char,
) -> std::io::Result<()> {
    match model {
        IonosphereModel::Klobuchar(KbModel {
            alpha,
            beta,
            region,
        }) => {
            writeln!(
                w,
                "{}{}{}",
                format_e19(alpha.0, exponent),
                format_e19(alpha.1, exponent),
                format_e19(alpha.2, exponent),
            )?;
            format_fields(w, &[alpha.3, beta.0, beta.1, beta.2], exponent)?;
            // the region code is a trailing value unless the
            // record header carries it as a subtype (RINEX 4.02)
            if *region == KbRegionCode::JapanArea && k.subtype.is_none() {
                format_fields(w, &[beta.3, 1.0], exponent)
            } else {
                format_fields(w, &[beta.3], exponent)
            }
        },
        IonosphereModel::NequickG(NgModel { a, region }) => {
            writeln!(
                w,
                "{}{}{}",
                format_e19(a.0, exponent),
                format_e19(a.1, exponent),
                format_e19(a.2, exponent),
            )?;
            format_fields(w, &[region.bits() as f64], exponent)
        },
        IonosphereModel::Bdgim(BdModel { alpha }) => {
            writeln!(
                w,
                "{}{}{}",
                format_e19(alpha.0, exponent),
                format_e19(alpha.1, exponent),
                format_e19(alpha.2, exponent),
            )?;
            format_fields(w, &[alpha.3, alpha.4, alpha.5, alpha.6], exponent)?;
            format_fields(w, &[alpha.7, alpha.8], exponent)
        },
        IonosphereModel::NavicKlobuchar(NavicKbModel {
            iodk,
            alpha,
            beta,
            longitude_deg,
            latitude_deg,
        }) => {
            writeln!(w, "{}", format_e19(*iodk, exponent))?;
            format_fields(w, &[alpha.0, alpha.1, alpha.2, alpha.3], exponent)?;
            format_fields(w, &[beta.0, beta.1, beta.2, beta.3], exponent)?;
            format_fields(
                w,
                &[
                    longitude_deg.0,
                    longitude_deg.1,
                    latitude_deg.0,
                    latitude_deg.1,
                ],
                exponent,
            )
        },
        IonosphereModel::NavicNequick(NavicNeqnModel { iodn, regions }) => {
            writeln!(w, "{}", format_e19(*iodn, exponent))?;
            for region in regions.iter() {
                format_fields(
                    w,
                    &[region.a.0, region.a.1, region.a.2, region.idf],
                    exponent,
                )?;
                format_fields(
                    w,
                    &[
                        region.longitude_deg.0,
                        region.longitude_deg.1,
                        region.modip_deg.0,
                        region.modip_deg.1,
                    ],
                    exponent,
                )?;
            }
            Ok(())
        },
        IonosphereModel::GlonassCdma(GloCdmaModel { c_a, c_f107, c_ap }) => {
            writeln!(
                w,
                "{}{}{}",
                format_e19(*c_a, exponent),
                format_e19(*c_f107, exponent),
                format_e19(*c_ap, exponent),
            )
        },
    }
}

/// Formats the navigation [Record] in the revision defined by [Header].
/// The record is browsed in key order, which is chronological and
/// then per satellite.
///
/// The record may have been parsed from another revision:
/// - written as RINEX 2 or 3, only the ephemerides of the legacy
///   messages are kept (LNAV, FDMA, INAV, FNAV, D1, D2, SBAS); the
///   modern messages (CNAV, CNV1 to CNV3, L1NV, L1OC, L3OC) and the
///   STO, EOP and ION records have no representation there and are
///   skipped. RINEX 2 files describe a single constellation: when the
///   header names one, the other satellites are skipped too.
/// - written as RINEX 4, ephemerides parsed from RINEX 2 or 3 are
///   given their RINEX 4 message type: FDMA for GLONASS, SBAS for
///   SBAS, INAV or FNAV for Galileo from the data source flags, D1 or
///   D2 for BeiDou from the GEO PRN range.
pub fn format<W: Write>(
    writer: &mut BufWriter<W>,
    rec: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    let version = header.version;
    let major = version.major;
    let exponent = if major < 3 { 'D' } else { 'E' };

    // RINEX 2: single constellation
    let constellation = if major < 3 {
        header.constellation.filter(|c| *c != Constellation::Mixed)
    } else {
        None
    };

    for (k, frame) in rec.iter() {
        if let Some(eph) = frame.as_ephemeris() {
            if major > 3 {
                let key = NavKey {
                    msgtype: v4_message_type(k, eph),
                    ..*k
                };
                format_epoch_v4(writer, &key)?;
                format_ephemeris(writer, &key, key.msgtype, eph, version, exponent)?;
            } else {
                if !is_legacy_message(k.msgtype, k.sv.constellation) {
                    continue;
                }
                if let Some(constellation) = constellation {
                    let same = if constellation.is_sbas() {
                        k.sv.constellation.is_sbas()
                    } else {
                        k.sv.constellation == constellation
                    };
                    if !same {
                        continue;
                    }
                }
                format_epoch_v2v3(writer, k, major)?;
                // every RINEX 2/3 orbit definition is filed as LNAV
                format_ephemeris(writer, k, NavMessageType::LNAV, eph, version, exponent)?;
            }
        } else if major > 3 {
            format_epoch_v4(writer, k)?;
            if let Some(sto) = frame.as_system_time() {
                format_system_time(writer, sto, exponent)?;
            } else if let Some(eop) = frame.as_earth_orientation() {
                format_earth_orientation(writer, eop, exponent)?;
            } else if let Some(model) = frame.as_ionosphere_model() {
                format_ionosphere_model(writer, k, model, exponent)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::{format_e19, format_epoch_v2v3, format_epoch_v4};
    use crate::navigation::{NavFrameType, NavKey, NavMessageSubtype, NavMessageType};
    use crate::prelude::{Constellation, Epoch, SV};
    use crate::tests::formatting::Utf8Buffer;
    use std::io::BufWriter;
    use std::str::FromStr;

    #[test]
    fn e19_formatting() {
        assert_eq!(format_e19(0.0, 'E'), " 0.000000000000E+00");
        assert_eq!(format_e19(7.874774746600E-04, 'D'), " 7.874774746600D-04");
        assert_eq!(format_e19(-5.911715561520E-12, 'D'), "-5.911715561520D-12");
        assert_eq!(format_e19(2.138E3, 'E'), " 2.138000000000E+03");
        assert_eq!(format_e19(-1.605716170161e-05, 'E'), "-1.605716170161E-05");
        assert_eq!(format_e19(5.184E5, 'E'), " 5.184000000000E+05");
        assert_eq!(format_e19(1.0, 'E'), " 1.000000000000E+00");
    }

    #[test]
    fn nav_fmt_v2v3() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-01-01T00:00:00 UTC").unwrap(),
            sv: SV::from_str("E01").unwrap(),
            frmtype: NavFrameType::from_str("EOP").unwrap(),
            msgtype: NavMessageType::from_str("LNAV").unwrap(),
            subtype: None,
        };

        format_epoch_v2v3(&mut writer, &key, 3).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(&utf8_ascii, "E01 2023 01 01 00 00 00");
    }

    #[test]
    fn nav_fmt_v2() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2021-01-01T02:00:00 GPST").unwrap(),
            sv: SV::from_str("G01").unwrap(),
            frmtype: NavFrameType::from_str("EPH").unwrap(),
            msgtype: NavMessageType::from_str("LNAV").unwrap(),
            subtype: None,
        };

        format_epoch_v2v3(&mut writer, &key, 2).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(&utf8_ascii, " 1 21  1  1  2  0  0.0");
    }

    #[test]
    fn navfmt_v4_ephemeris() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-03-12T00:00:00 UTC").unwrap(),
            sv: SV::from_str("G01").unwrap(),
            frmtype: NavFrameType::from_str("EPH").unwrap(),
            msgtype: NavMessageType::from_str("LNAV").unwrap(),
            subtype: None,
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> EPH G01 LNAV
G01 2023 03 12 00 00 00"
        );
    }

    #[test]
    fn navfmt_v4_iono() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-03-12T00:08:54 UTC").unwrap(),
            sv: SV::from_str("G12").unwrap(),
            frmtype: NavFrameType::from_str("ION").unwrap(),
            msgtype: NavMessageType::from_str("LNAV").unwrap(),
            subtype: None,
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> ION G12 LNAV
    2023 03 12 00 08 54"
        );
    }

    #[test]
    fn navfmt_v4_iono_subtype() {
        // RINEX 4.02: message subtype as fourth field
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-06-24T00:07:30 UTC").unwrap(),
            sv: SV::from_str("I10").unwrap(),
            frmtype: NavFrameType::from_str("ION").unwrap(),
            msgtype: NavMessageType::from_str("CNVX").unwrap(),
            subtype: Some(NavMessageSubtype::KLOB),
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> ION I10 CNVX KLOB
    2023 06 24 00 07 30"
        );
    }

    #[test]
    fn navfmt_v4_blank_prn() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2020-09-15T00:00:00 UTC").unwrap(),
            sv: SV::new(Constellation::Galileo, 0),
            frmtype: NavFrameType::from_str("STO").unwrap(),
            msgtype: NavMessageType::from_str("IFNV").unwrap(),
            subtype: None,
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> STO E   IFNV
    2020 09 15 00 00 00"
        );
    }

    #[test]
    fn navfmt_v4_systime() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-03-12T00:08:54 UTC").unwrap(),
            sv: SV::from_str("G12").unwrap(),
            frmtype: NavFrameType::from_str("STO").unwrap(),
            msgtype: NavMessageType::from_str("LNAV").unwrap(),
            subtype: None,
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> STO G12 LNAV
    2023 03 12 00 08 54"
        );
    }

    #[test]
    fn navfmt_v4_eop() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-03-12T00:08:54 UTC").unwrap(),
            sv: SV::from_str("G12").unwrap(),
            frmtype: NavFrameType::from_str("EOP").unwrap(),
            msgtype: NavMessageType::from_str("CNVX").unwrap(),
            subtype: None,
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> EOP G12 CNVX
    2023 03 12 00 08 54"
        );
    }
}
