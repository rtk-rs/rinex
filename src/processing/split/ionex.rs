use crate::{
    ionex::Record,
    prelude::{Duration, Epoch},
};

pub fn split(rec: &Record, t: Epoch) -> (Record, Record) {
    let before = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch < t {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    let after = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch >= t {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    (before, after)
}

pub fn split_mut(rec: &mut Record, t: Epoch) -> Record {
    let after = rec
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

    after
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

        pending.insert(k.clone(), v.clone());
    }

    ret
}
