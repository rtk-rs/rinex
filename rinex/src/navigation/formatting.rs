use itertools::Itertools;

use std::io::{BufWriter, Write};

use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    error::FormattingError,
    navigation::{NavFrameType, NavKey, Record},
    prelude::Header,
};

fn format_epoch_v2v3<W: Write>(w: &mut BufWriter<W>, k: &NavKey) -> std::io::Result<()> {
    let (yyyy, m, d, hh, mm, ss, _) = epoch_decomposition(k.epoch);
    write!(
        w,
        "{:x} {:04} {:02} {:02} {:02} {:02} {:02}",
        k.sv, yyyy, m, d, hh, mm, ss
    )
}

fn format_epoch_v4<W: Write>(w: &mut BufWriter<W>, k: &NavKey) -> std::io::Result<()> {
    let (yyyy, m, d, hh, mm, ss, _) = epoch_decomposition(k.epoch);

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

    match k.frmtype {
        NavFrameType::Ephemeris => {
            write!(
                w,
                "\n{:x} {:04} {:02} {:02} {:02} {:02} {:02}",
                k.sv, yyyy, m, d, hh, mm, ss
            )
        },
        _ => {
            write!(
                w,
                "\n        {:04} {:02} {:02} {:02} {:02} {:02}",
                yyyy, m, d, hh, mm, ss
            )
        },
    }
}

// /*
//  * When formatting floating point number in Navigation RINEX,
//  * exponent are expected to be in the %02d form,
//  * but Rust is only capable of formating %d (AFAIK).
//  * With this macro, we simply rework all exponents encountered in a string
//  */
// fn double_exponent_digits(content: &str) -> String {
//     // replace "eN " with "E+0N"
//     let re = Regex::new(r"e\d{1} ").unwrap();
//     let lines = re.replace_all(content, |caps: &Captures| format!("E+0{}", &caps[0][1..]));
//
//     // replace "eN" with "E+0N"
//     let re = Regex::new(r"e\d{1}").unwrap();
//     let lines = re.replace_all(&lines, |caps: &Captures| format!("E+0{}", &caps[0][1..]));
//
//     // replace "e-N " with "E-0N"
//     let re = Regex::new(r"e-\d{1} ").unwrap();
//     let lines = re.replace_all(&lines, |caps: &Captures| format!("E-0{}", &caps[0][2..]));
//
//     // replace "e-N" with "e-0N"
//     let re = Regex::new(r"e-\d{1}").unwrap();
//     let lines = re.replace_all(&lines, |caps: &Captures| format!("E-0{}", &caps[0][2..]));
//
//     lines.to_string()
// }

// /*
//  * Reworks generated/formatted line to match standards
//  */
// fn fmt_rework(major: u8, lines: &str) -> String {
//     /*
//      * There's an issue when formatting the exponent 00 in XXXXX.E00
//      * Rust does not know how to format an exponent on multiples digits,
//      * and RINEX expects two.
//      * If we try to rework this line, it may corrupt some SVNN fields.
//      */
//     let mut lines = double_exponent_digits(lines);
//
//     if major < 3 {
//         /*
//          * In old RINEX, D+00 D-01 is used instead of E+00 E-01
//          */
//         lines = lines.replace("E-", "D-");
//         lines = lines.replace("E+", "D+");
//     }
//     lines.to_string()
// }

pub fn format<W: Write>(
    writer: &mut BufWriter<W>,
    rec: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    let v4 = header.version.major > 3;

    // timeframe in chronological order
    for epoch in rec.iter().map(|(k, _v)| k.epoch).unique().sorted() {
        // per SV sorted
        for sv in rec
            .iter()
            .filter_map(|(k, _v)| if k.epoch == epoch { Some(k.sv) } else { None })
            .unique()
            .sorted()
        {
            // per sorted frame type
            for frmtype in rec
                .iter()
                .filter_map(|(k, _v)| {
                    if k.epoch == epoch && k.sv == sv {
                        Some(k.frmtype)
                    } else {
                        None
                    }
                })
                .unique()
                .sorted()
            {
                if let Some((k, v)) = rec
                    .iter()
                    .filter(|(k, _v)| k.epoch == epoch && k.sv == sv && k.frmtype == frmtype)
                    .reduce(|k, _| k)
                {
                    if v4 {
                        format_epoch_v4(writer, k)?;
                    } else {
                        format_epoch_v2v3(writer, k)?;
                    }
                    if let Some(eph) = v.as_ephemeris() {
                    } else if let Some(eop) = v.as_earth_orientation() {
                    } else if let Some(sto) = v.as_system_time() {
                    } else if let Some(ion) = v.as_ionosphere_model() {
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::{format_epoch_v2v3, format_epoch_v4};
    use crate::navigation::{NavFrameType, NavKey, NavMessageSubtype, NavMessageType};
    use crate::prelude::{Constellation, Epoch, SV};
    use crate::tests::formatting::Utf8Buffer;
    use std::io::BufWriter;
    use std::str::FromStr;

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

        format_epoch_v2v3(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(&utf8_ascii, "E01 2023 01 01 00 00 00");
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
            epoch: Epoch::from_str("2023-03-12T00:20:00 UTC").unwrap(),
            sv: SV::from_str("C21").unwrap(),
            frmtype: NavFrameType::from_str("STO").unwrap(),
            msgtype: NavMessageType::from_str("CNVX").unwrap(),
            subtype: None,
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> STO C21 CNVX
        2023 03 12 00 20 00"
        );
    }

    #[test]
    fn navfmt_v4_eop() {
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-03-14T16:51:12 UTC").unwrap(),
            sv: SV::from_str("G27").unwrap(),
            frmtype: NavFrameType::from_str("EOP").unwrap(),
            msgtype: NavMessageType::from_str("CNVX").unwrap(),
            subtype: None,
        };

        format_epoch_v4(&mut writer, &key).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(
            &utf8_ascii,
            "> EOP G27 CNVX
        2023 03 14 16 51 12"
        );
    }
}
