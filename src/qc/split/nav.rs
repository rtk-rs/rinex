use crate::{
    navigation::Record,
    prelude::{Duration, Epoch},
};

pub fn split(rec: &Record, epoch: Epoch) -> (Record, Record) {
    let r0 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch < epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();
    let r1 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch >= epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();
    (r0, r1)
}

pub fn split_mut(rec: &mut Record, t: Epoch) -> Record {
    let r1 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch >= t {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    rec.retain(|k, _| k.epoch < t);
    r1
}

pub fn split_even_dt(rec: &Record, dt: Duration) -> Vec<Record> {
    let mut pending = Record::new();
    let mut ret = Vec::<Record>::new();

    let mut t0 = Option::<Epoch>::None;

    for (k, v) in rec.iter() {
        if let Some(t) = t0 {
            if k.epoch > t + dt {
                // reset: new chunk
                t0 = Some(k.epoch);
                ret.push(pending);
                pending = Record::new();
            }
        } else {
            t0 = Some(k.epoch);
        }

        pending.insert(*k, v.clone());
    }

    ret
}
