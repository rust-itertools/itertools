#![cfg(feature = "use_std")]

use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Iterator;

/// Return a `HashMap` of keys mapped to a list of their corresponding values.
///
/// See [`.into_group_map()`](../trait.Itertools.html#method.into_group_map)
/// for more information.
pub fn into_group_map<I, K, V>(iter: I) -> HashMap<K, Vec<V>>
    where I: Iterator<Item=(K, V)>,
          K: Hash + Eq,
{
    let mut lookup = HashMap::new();

    for (key, val) in iter {
        lookup.entry(key).or_insert(Vec::new()).push(val);
    }

    lookup
}

pub fn into_group_map_by<I, K, V>(iter: I, f: impl Fn(&V) -> K) -> HashMap<K, Vec<V>>
    where I: Iterator<Item=V>,
          K: Hash + Eq,
{
    let mut lookup = HashMap::new();

    for val in iter {
        let key = f(&val);
        lookup.entry(key).or_insert(Vec::new()).push(val);
    }

    lookup
}

pub fn into_group_map_by_fold<I, K, V, Acc, Fold>(iter: I, f_key: impl Fn(&V) -> K,
                                             init: Acc,
                                             fold: Fold) -> HashMap<K, Acc>
    where I: Iterator<Item=V>,
          K: Hash + Eq,
          Acc: Clone,
          Fold : FnMut(Acc,V) -> Acc + Clone


{
    let mut lookup = HashMap::new();

    for val in iter {
        let key = f_key(&val);
        lookup.entry(key).or_insert(Vec::new()).push(val);
    }

    lookup
        .into_iter()
        .map(move |(key,value)| (key, value.into_iter().fold(init.clone(), fold.clone())))
        .collect()
}

