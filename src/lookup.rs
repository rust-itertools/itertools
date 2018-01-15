#![cfg(feature = "use_std")]

use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Iterator;

/// Return a `HashMap` of keys mapped to a list of their corresponding values.
///
/// See [`.to_group_lookup()`](../trait.Itertools.html#method.to_group_lookup)
/// for more information.
pub fn to_group_lookup<I, K, V>(iter: I) -> HashMap<K, Vec<V>>
    where I: Iterator<Item=(K, V)>,
          K: Hash + Eq,
{
    let mut lookup = HashMap::new();

    for (key, val) in iter {
        lookup.entry(key).or_insert(Vec::new()).push(val);
    }

    lookup
}