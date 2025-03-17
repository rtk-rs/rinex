//! Ephemeris message formatting
use crate::{
    navigation::{orbits::closest_nav_standards, Ephemeris, NavMessageType},
    prelude::{Constellation, SV},
    FormattingError, Version,
};

use std::io::{BufWriter, Write};

impl Ephemeris {
    /// Formats [Ephemeris] according to RINEX standards
    pub(crate) fn format<W: Write>(
        &self,
        w: &mut BufWriter<W>,
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
        write!(
            w,
            "{:.17E} {:.17E} {:.17E}\n",
            self.clock_bias, self.clock_drift, self.clock_drift_rate
        )?;

        // following standard specs
        let data_fields = &standard_specs.items;
        for i in 0..data_fields.len() {
            write!(w, "0.000000000000D+00")?;
            if let Some(value) = self.get_orbit_f64(data_fields[i].0) {
                write!(w, "0.000000000000D+00")?;
            } else {
                // standardized missing field
                write!(w, "0.000000000000D+00")?;
            }
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

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));

        ephemeris.format(&mut buf, g01, version, msgtype).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content, " 17  1  1  0  0  0.0000000  0 10G03G08G14G16G22G23G26G27G31G32\n",
            "-5.154609680176e-04-6.708145150469e-11 0.000000000000e+00
     1.000000000000e+00-4.140000000000e+02-3.140000000000e-09-1.101700000000e+00
    -1.366203000000e-05"
        );
    }
}
