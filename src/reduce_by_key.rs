#![cfg(feature = "use_std")]

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::iter::Iterator;

/// Return a `HashMap` of keys mapped to a reduce merge of their values.
/// See [`.reduce_by_key()`](../trait.Itertools.html#method.reduce_by_key)
/// for more information.
pub fn reduce_by_key<I, K, V, F>(iter: I, mut f: F) -> HashMap<K, V>
where
    I: Iterator<Item = (K, V)>,
    K: Hash + Eq,
    F: FnMut(&mut V, V),
{
    let mut lookup = HashMap::new();

    for (k, v) in iter {
        match lookup.entry(k) {
            Entry::Occupied(e) => f(e.into_mut(), v),
            Entry::Vacant(e) => {
                e.insert(v);
            }
        }
    }

    lookup
}
