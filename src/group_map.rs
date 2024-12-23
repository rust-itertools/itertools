#![cfg(feature = "use_std")]

use core::hash::BuildHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Iterator;

/// Return a `HashMap` of keys mapped to a list of their corresponding values.
///
/// See [`.into_group_map()`](crate::Itertools::into_group_map)
/// for more information.
pub fn into_group_map_with_hasher<I, K, V, S>(iter: I, hash_builder: S) -> HashMap<K, Vec<V>, S>
where
    I: Iterator<Item = (K, V)>,
    K: Hash + Eq,
    S: BuildHasher,
{
    let mut lookup = HashMap::with_hasher(hash_builder);

    iter.for_each(|(key, val)| {
        lookup.entry(key).or_insert_with(Vec::new).push(val);
    });

    lookup
}

pub fn into_group_map_by_with_hasher<I, K, V, F, S>(
    iter: I,
    mut f: F,
    hash_builder: S,
) -> HashMap<K, Vec<V>, S>
where
    I: Iterator<Item = V>,
    K: Hash + Eq,
    F: FnMut(&V) -> K,
    S: BuildHasher,
{
    into_group_map_with_hasher(iter.map(|v| (f(&v), v)), hash_builder)
}
