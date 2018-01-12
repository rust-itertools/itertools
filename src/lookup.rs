#![cfg(feature = "use_std")]

use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Iterator;

/// Return a `HashMap` of keys mapped to a list of their corresponding values,
/// as determined by the specified function.
///
/// See [`.to_lookup()`](../trait.Itertools.html#method.to_lookup)
/// for more information.
pub fn to_lookup<I, K, F>(iter: I, get_key: F) -> HashMap<K, Vec<I::Item>>
    where I: Iterator,
          K: Hash + Eq,
          F: Fn(&I::Item) -> K
{
    let mut lookup = HashMap::new();

    for val in iter {
        let key = get_key(&val);
        let slot = lookup.entry(key).or_insert(Vec::new());
        slot.push(val);
    }

    lookup
}