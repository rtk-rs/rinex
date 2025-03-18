//! Ephemeris message formatting
use crate::{
    navigation::{
        formatting::ascii::AsciiString, orbits::closest_nav_standards, Ephemeris, NavMessageType,
    },
    prelude::{Constellation, SV},
    FormattingError, Version,
};

use std::io::{BufWriter, Write};

fn format_value(value: f64) -> String {
    let is_signed = value.is_sign_negative();
    let trunc = value.trunc() as u32;
    if value.is_sign_negative() {
        format!("{:19.12E}", value)
    } else if value == 0.0 {
        "0.000000000000E0".to_string()
    } else {
        if trunc < 10 {
            format!(" {:19.12E}", value)
        } else {
            format!("{:19.12E}", value)
        }
    }
}

impl Ephemeris {
    /// Formats [Ephemeris] according to RINEX standards
    pub(crate) fn format(
        &self,
        string: &mut String,
        sv: SV,
        version: Version,
        msgtype: NavMessageType,
    ) -> Result<(), FormattingError> {
        let sv_constellation = if sv.constellation.is_sbas() {
            Constellation::SBAS
        } else {
            sv.constellation
        };

        // retrieve standard specs
        let standard_specs = match closest_nav_standards(sv_constellation, version, msgtype) {
            Some(specs) => specs,
            None => {
                return Err(FormattingError::MissingNavigationStandards);
            },
        };

        // starts with (clock_bias, drift, rate)
        // epoch has already been buffered
        let formatted = format!(
            "{} {} {}\n",
            format_value(self.clock_bias),
            format_value(self.clock_drift),
            format_value(self.clock_drift_rate),
        );

        let ascii = AsciiString::from_str(&formatted);
        string.push_str(&ascii.to_string());

        // following standard specs
        let data_fields = &standard_specs.items;
        for i in 0..data_fields.len() {
            let formatted = if let Some(value) = self.get_orbit_f64(data_fields[i].0) {
                if (i % 4) == 0 {
                    format!("  {}", format_value(value))
                } else {
                    format_value(value)
                }
            } else {
                if (i % 4) == 0 {
                    format!("   {}", format_value(0.0))
                } else {
                    format_value(0.0)
                }
            };

            let ascii = AsciiString::from_str(&formatted);
            string.push_str(&ascii.to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::navigation::{Ephemeris, NavMessageType, OrbitItem};
    use crate::prelude::{Version, SV};

    use std::io::BufWriter;
    use std::str::FromStr;

    use crate::tests::formatting::Utf8Buffer;

    #[test]
    fn test_value_formatter() {}

    #[test]
    fn ephemeris_formatting() {
        let g01 = SV::from_str("G01").unwrap();
        let version = Version::from_str("2.0").unwrap();
        let msgtype = NavMessageType::LNAV;

        let ephemeris = Ephemeris {
            clock_bias: -5.15460980176e-4,
            clock_drift: -6.708145150469e-11,
            clock_drift_rate: 0.0,
            orbits: [
                ("1".to_string(), OrbitItem::F64(1.0)),
                ("2".to_string(), OrbitItem::F64(-4.14e2)),
                ("3".to_string(), OrbitItem::F64(-3.14e-9)),
                ("4".to_string(), OrbitItem::F64(-1.1017)),
                ("5".to_string(), OrbitItem::F64(-1.366203)),
            ]
            .into_iter()
            .collect(),
        };

        let mut content = String::new();

        ephemeris
            .format(&mut content, g01, version, msgtype)
            .unwrap();

        assert_eq!(
            content,
            " -5.154609680176E-04-6.708145150469E-11 0.000000000000E+00
     1.000000000000E+00-4.140000000000E+02-3.140000000000E-09-1.101700000000E+00
    -1.366203000000E-05"
        );
    }
}
