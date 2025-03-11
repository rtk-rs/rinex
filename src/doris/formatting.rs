use crate::{
    doris::{ClockObservation, DorisKey, Record},
    epoch::epoch_decompose as epoch_decomposition,
    prelude::{FormattingError, Header},
};

use std::io::{BufWriter, Write};

fn format_epoch<W: Write>(
    w: &mut BufWriter<W>,
    k: &DorisKey,
    clock_extrapolated: bool,
    clock: &ClockObservation,
) -> Result<(), FormattingError> {
    let (y, m, d, hh, mm, ss, nanos) = epoch_decomposition(k.epoch);

    write!(
        w,
        "> {:04} {:02} {:02} {:02} {:02} {:02}.{:07} {:3} {:3} {:14.3}",
        y,
        m,
        d,
        hh,
        mm,
        ss,
        nanos / 100,
        clock_extrapolated,
        k.flag,
        clock.offset_s,
    )?;

    Ok(())
}

pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    const NUM_OBS_PER_LINE: usize = 5;
    const OBSERVATIONS_BLANK: &str = "              ";

    let header = header
        .doris
        .as_ref()
        .ok_or(FormattingError::UndefinedObservables)?;

    let _stations = &header.stations;

    for (k, v) in record.iter() {
        format_epoch(w, &k, v.clock_extrapolated, &v.clock)?;

        for station in header.stations.iter() {
            let mut modulo = 0;

            // following header specs
            for (nth, obs) in header.observables.iter().enumerate() {
                if nth == 0 {
                    write!(w, "D{:02}", station.key)?;
                }

                if let Some((_, signal)) = v
                    .signals
                    .iter()
                    .filter(|(k, _)| &k.station == station && &k.observable == obs)
                    .reduce(|k, _| k)
                {
                    write!(w, "{:14.3}", signal.value)?;
                    if let Some(flag) = signal.m1 {
                        write!(w, "{}", flag)?;
                    } else {
                        write!(w, "{}", ' ')?;
                    }
                    if let Some(flag) = signal.m2 {
                        write!(w, "{}", flag)?;
                    } else {
                        write!(w, "{}", ' ')?;
                    }
                } else {
                    write!(w, "{}", OBSERVATIONS_BLANK)?;
                }

                if (nth % NUM_OBS_PER_LINE) == NUM_OBS_PER_LINE - 1 {
                    write!(w, "{}   \n", ' ')?;
                }

                modulo = nth % NUM_OBS_PER_LINE;
            }

            if modulo != NUM_OBS_PER_LINE - 1 {
                write!(w, "{}", '\n')?;
            }
        }
    }

    Ok(())
}
