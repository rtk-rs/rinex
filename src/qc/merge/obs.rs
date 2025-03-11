use crate::{observation::Record, prelude::qc::MergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (k, rhs) in rhs.iter() {
        if let Some(lhs) = rec.get_mut(k) {
            // TODO: could merge clock field
            //  but only if receivers do match exactly !
            for rhs in rhs.signals.iter() {
                if let Some(lhs) = lhs
                    .signals
                    .iter_mut()
                    .find(|sig| sig.sv == rhs.sv && sig.observable == rhs.observable)
                {
                    if let Some(lli) = rhs.lli {
                        if lhs.lli.is_none() {
                            lhs.lli = Some(lli);
                        }
                    }
                    if let Some(snr) = rhs.snr {
                        if lhs.snr.is_none() {
                            lhs.snr = Some(snr);
                        }
                    }
                } else {
                    lhs.signals.push(rhs.clone());
                }
            }
        } else {
            rec.insert(*k, rhs.clone());
        }
    }
    Ok(())
}
