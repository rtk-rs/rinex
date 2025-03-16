use crate::{
    clock::Record,
    prelude::{Duration, Epoch},
};

pub fn split(rec: &Record, epoch: Epoch) -> (Record, Record) {
    let r0 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k <= &epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();
    let r1 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k > &epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();
    (r0, r1)
}

pub fn split_mut(_rec: &mut Record, _t: Epoch) -> Record {
    Record::default()
}

pub fn split_even_dt(_rec: &Record, _duration: Duration) -> Vec<Record> {
    Vec::new()
}
