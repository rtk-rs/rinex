//! RINEX File merging (combination)
use crate::prelude::Epoch;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::Hash;

/*
 * Appends given vector into self.
 */
pub(crate) fn merge_mut_vec<T: Clone>(lhs: &mut Vec<T>, rhs: &Vec<T>) {
    for item in rhs {
        lhs.push(item.clone());
    }
}

/*
 * Merges given vector into self, but ensures values are unique.
 */
pub(crate) fn merge_mut_unique_vec<T: Clone + PartialEq>(lhs: &mut Vec<T>, rhs: &Vec<T>) {
    for item in rhs {
        if !lhs.contains(item) {
            lhs.push(item.clone());
        }
    }
}

/*
 * Merges given map into self but ensures both keys and values are unique.
 */
pub(crate) fn merge_mut_unique_map2d<K: PartialEq + Eq + Hash + Clone, V: Clone + PartialEq>(
    lhs: &mut HashMap<K, Vec<V>>,
    rhs: &HashMap<K, Vec<V>>,
) {
    for (k, values) in rhs.iter() {
        if let Some(vvalues) = lhs.get_mut(k) {
            for value in values {
                if !vvalues.contains(value) {
                    vvalues.push(value.clone());
                }
            }
        } else {
            lhs.insert(k.clone(), values.clone());
        }
    }
}

/*
 * Merges optionnal data fields, rhs overwrites lhs, only if lhs is not previously defined.
 */
pub(crate) fn merge_mut_option<T: Clone>(lhs: &mut Option<T>, rhs: &Option<T>) {
    if lhs.is_none() {
        if let Some(rhs) = rhs {
            *lhs = Some(rhs.clone());
        }
    }
}

/*
 * Merges "TIME OF FIRST" special OBSERVATION header field
 */
pub(crate) fn merge_time_of_first_obs(lhs: &mut Option<Epoch>, rhs: &Option<Epoch>) {
    if lhs.is_none() {
        if let Some(rhs) = rhs {
            *lhs = Some(*rhs);
        }
    } else if let Some(rhs) = rhs {
        let tl = lhs.unwrap();
        *lhs = Some(std::cmp::min(tl, *rhs));
    }
}

/*
 * Merges "TIME OF LAST" special OBSERVATION header field
 */
pub(crate) fn merge_time_of_last_obs(lhs: &mut Option<Epoch>, rhs: &Option<Epoch>) {
    if lhs.is_none() {
        if let Some(rhs) = rhs {
            *lhs = Some(*rhs);
        }
    } else if let Some(rhs) = rhs {
        let tl = lhs.unwrap();
        *lhs = Some(std::cmp::max(tl, *rhs));
    }
}
