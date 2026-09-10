use std::io::{BufWriter, Write};

use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    error::FormattingError,
    navigation::{NavFrame, NavFrameType, NavKey, Record},
    prelude::{Constellation, Epoch, Header},
};

pub(crate) struct NavFormatter {
    value: f64,
    width: usize,
    precision: usize,
    /// Exponent letter: 'E', or 'D' in RINEX 2 records
    exponent: char,
}

impl NavFormatter {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            width: 15,
            precision: 12,
            exponent: 'E',
        }
    }

    /// Same formatting, with a 'D' exponent letter as found in
    /// RINEX 2 navigation records.
    pub fn new_v2(value: f64) -> Self {
        Self {
            exponent: 'D',
            ..Self::new(value)
        }
    }

    pub fn new_iono_alpha_beta(value: f64) -> Self {
        Self {
            value,
            width: 3,
            precision: 4,
            exponent: 'E',
        }
    }

    pub fn new_time_system_correction_v2(value: f64) -> Self {
        Self {
            value,
            width: 17,
            precision: 12,
            exponent: 'E',
        }
    }

    pub fn new_time_system_correction_v3_offset(value: f64) -> Self {
        Self {
            value,
            width: 14,
            precision: 10,
            exponent: 'E',
        }
    }

    pub fn new_time_system_correction_v3_drift(value: f64) -> Self {
        Self {
            value,
            width: 13,
            precision: 9,
            exponent: 'E',
        }
    }
}

impl std::fmt::Display for NavFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = self.value;
        let sign_str = if value.is_sign_positive() { " " } else { "" };
        let formatted = format!(
            "{:width$.precision$E}",
            value,
            width = self.width,
            precision = self.precision
        );

        // reformat exponent
        let parts = formatted.split('E').collect::<Vec<_>>();

        if parts.len() == 2 {
            let (base, exponent) = (parts[0], parts[1]);
            let exp_sign = if exponent.starts_with('-') { "-" } else { "+" };
            let exp_value = exponent
                .trim_start_matches(&['+', '-'][..])
                .parse::<i32>()
                .unwrap();
            let formatted_exponent = format!("{}{:02}", exp_sign, exp_value);
            write!(
                f,
                "{}{}{}{}",
                sign_str, base, self.exponent, formatted_exponent
            )
        } else {
            write!(f, "{}", formatted)
        }
    }
}

fn format_epoch_v2v3<W: Write>(
    w: &mut BufWriter<W>,
    k: &NavKey,
    v2: bool,
    file_constell: &Constellation,
) -> std::io::Result<()> {
    let (yyyy, m, d, hh, mm, ss, nanos) = epoch_decomposition(k.epoch);

    let decis = nanos / 100_000;

    if v2 && *file_constell != Constellation::Mixed {
        // RINEX 2: PRN, two digit year, month, day, hour and minute
        // as I2, seconds as F5.1
        write!(
            w,
            "{:2} {:2} {:2} {:2} {:2} {:2} {:2}.{:01}",
            k.sv.prn,
            yyyy % 100,
            m,
            d,
            hh,
            mm,
            ss,
            decis
        )
    } else {
        write!(
            w,
            "{:x} {:04} {:02} {:02} {:02} {:02} {:02}",
            k.sv, yyyy, m, d, hh, mm, ss
        )
    }
}

/// Formats the "yyyy mm dd hh mm ss" epoch of a RINEX 4 record.
pub(crate) fn format_epoch_v4_fields(epoch: Epoch) -> String {
    let (yyyy, m, d, hh, mm, ss, _) = epoch_decomposition(epoch);
    format!(
        "{:04} {:02} {:02} {:02} {:02} {:02}",
        yyyy, m, d, hh, mm, ss
    )
}

/// Formats the record header of a RINEX 4 record.
/// The ephemeris epoch line follows, the other records write their
/// own epoch line along with their data.
fn format_epoch_v4<W: Write>(w: &mut BufWriter<W>, k: &NavKey) -> std::io::Result<()> {
    // records naming the constellation only carry PRN 0
    let sv = if k.sv.prn == 0 {
        format!("{:x}  ", k.sv.constellation)
    } else {
        format!("{:x}", k.sv)
    };

    // message type, and RINEX 4.02 subtype when defined
    let msgtype = match k.subtype {
        Some(subtype) => format!("{} {}", k.msgtype, subtype),
        None => k.msgtype.to_string(),
    };

    match k.frmtype {
        NavFrameType::Ephemeris => {
            write!(
                w,
                "> EPH {} {}\n{} {}",
                sv,
                msgtype,
                sv,
                format_epoch_v4_fields(k.epoch)
            )
        },
        frmtype => writeln!(w, "> {} {} {}", frmtype, sv, msgtype),
    }
}

pub fn format<W: Write>(
    writer: &mut BufWriter<W>,
    rec: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    let version = header.version;

    let v2 = version.major < 3;
    let v4 = version.major > 3;

    let file_constell = header
        .constellation
        .ok_or(FormattingError::UndefinedConstellation)?;

    // the record is sorted by key: chronological order, then vehicle,
    // then message type. Every entry is written, including messages of
    // different types sharing an epoch and vehicle.
    for (k, v) in rec.iter() {
        if v4 {
            format_epoch_v4(writer, k)?;
        } else {
            match v {
                NavFrame::EPH(_) => format_epoch_v2v3(writer, k, v2, &file_constell)?,
                // no record representation before RINEX 4
                _ => continue,
            }
        }

        match v {
            NavFrame::EPH(eph) => eph.format(writer, k.sv, version, k.msgtype)?,
            NavFrame::STO(sto) => sto.format_v4(writer)?,
            NavFrame::EOP(eop) => eop.format_v4(writer, k.epoch)?,
            NavFrame::ION(model) => model.format_v4(writer, k.epoch)?,
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use super::{format_epoch_v2v3, format_epoch_v4, NavFormatter};
    use crate::navigation::{NavFrameType, NavKey, NavMessageType};
    use crate::prelude::{Constellation, Epoch, SV};
    use crate::tests::formatting::Utf8Buffer;
    use std::io::BufWriter;
    use std::str::FromStr;

    #[test]
    fn nav_formatter() {
        for (value, expected) in [
            (0.0, " 0.000000000000E+00"),
            (1.0, " 1.000000000000E+00"),
            (9.9, " 9.900000000000E+00"),
            (-1.0, "-1.000000000000E+00"),
            (-10.0, "-1.000000000000E+01"),
            (-0.123, "-1.230000000000E-01"),
            (0.123, " 1.230000000000E-01"),
        ] {
            let formatted = NavFormatter::new(value);
            assert_eq!(formatted.to_string(), expected);
        }

        // RINEX 2 records: 'D' exponent
        for (value, expected) in [
            (0.0, " 0.000000000000D+00"),
            (-1.0e-4, "-1.000000000000D-04"),
            (5.153693731310e+03, " 5.153693731310D+03"),
        ] {
            let formatted = NavFormatter::new_v2(value);
            assert_eq!(formatted.to_string(), expected);
        }
    }

    #[test]
    fn system_time_corr_v2_formatter() {
        for (value, expected) in [(-1.862645149231E-09, "-1.862645149231E-09")] {
            let formatted = NavFormatter::new_time_system_correction_v2(value);
            assert_eq!(formatted.to_string(), expected);
        }
    }

    #[test]
    fn system_time_corr_v3_formatter() {
        for (value, expected) in [
            (0.0000000000E+00, " 0.0000000000E+00"),
            (2.1536834538E-09, " 2.1536834538E-09"),
        ] {
            let formatted = NavFormatter::new_time_system_correction_v3_offset(value);
            assert_eq!(formatted.to_string(), expected);
        }

        for (value, expected) in [
            (-3.019806627E-14, "-3.019806627E-14"),
            (-9.769962617E-15, "-9.769962617E-15"),
        ] {
            let formatted = NavFormatter::new_time_system_correction_v3_drift(value);
            assert_eq!(formatted.to_string(), expected);
        }
    }

    #[test]
    fn nav_fmt_v2v3() {
        let gal = Constellation::Galileo;
        let buf = Utf8Buffer::new(1024);
        let mut writer = BufWriter::new(buf);

        let key = NavKey {
            epoch: Epoch::from_str("2023-01-01T00:00:00 UTC").unwrap(),
            sv: SV::from_str("E01").unwrap(),
            frmtype: NavFrameType::from_str("EOP").unwrap(),
            msgtype: NavMessageType::from_str("LNAV").unwrap(),
            subtype: None,
        };

        format_epoch_v2v3(&mut writer, &key, true, &gal).unwrap();

        let inner = writer.into_inner().unwrap();

        let utf8_ascii = inner.to_ascii_utf8();

        assert_eq!(&utf8_ascii, " 1 23  1  1  0  0  0.0");
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

        // the model writes the epoch line along with its data
        assert_eq!(&utf8_ascii, "> ION G12 LNAV\n");
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

        assert_eq!(&utf8_ascii, "> STO C21 CNVX\n");
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

        assert_eq!(&utf8_ascii, "> EOP G27 CNVX\n");
    }
}
