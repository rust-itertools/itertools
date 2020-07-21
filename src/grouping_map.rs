#![cfg(feature = "use_std")]

use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Iterator;

/// Creates a new `GroupingMap` from `iter`
pub fn new<I, K, V>(iter: I) -> GroupingMap<I>
where
    I: Iterator<Item = (K, V)>,
    K: Hash + Eq,
{
    GroupingMap { iter }
}

/// `GroupingMap` is an intermediate struct for efficient "group-and-fold" operations.
/// It groups elements by their key and at the same time fold each group
/// using some aggregating operation.
/// 
/// No method on this struct performs temporary allocations.
/// 
// See [`.into_grouping_map()`](../trait.Itertools.html#method.into_grouping_map)
// for more information.
pub struct GroupingMap<I> {
    iter: I,
}

impl<I, K, V> GroupingMap<I>
where
    I: Iterator<Item = (K, V)>,
    K: Hash + Eq,
{
    /// Groups elements from the `GroupingMap` source by key and applies `operation` to the elements
    /// of each group sequentially, passing the key, the previously accumulated value
    /// and the current element as arguments, and stores the results in an `HashMap`.
    ///
    /// The `operation` function is invoked on each element with the following parameters:
    ///  - a reference to the key of the group this element belongs to;
    ///  - the current value of the accumulator of the group or `None` if it's the first element
    ///    encountered in the group;
    ///  - the element from the source being aggregated;
    /// If `operation` returns `Some(element)` then the accumulator is updated with `element`,
    /// otherwise the previous accumulation is discarded.
    ///
    /// Return a `HashMap` associating the key of each group with the result of aggregation of the group elements.
    ///
    /// This is the generic way to perform any operations on a `Grouping`.
    /// It's suggested to use it only to implement custom operations when the already provided ones are not enough.
    pub fn aggregate<FO, R>(self, mut operation: FO) -> HashMap<K, R>
    where
        FO: FnMut(Option<R>, &K, V) -> Option<R>,
    {
        let mut destination_map = HashMap::new();

        for (key, val) in self.iter {
            let acc = destination_map.remove(&key);
            if let Some(op_res) = operation(acc, &key, val) {
                destination_map.insert(key, op_res);
            }
        }

        destination_map
    }

    pub fn fold<FO, R>(self, init: R, mut operation: FO) -> HashMap<K, R>
    where
        R: Clone,
        FO: FnMut(R, &K, V) -> R,
    {
        self.aggregate(|acc, key, val| {
            let acc = acc.unwrap_or_else(|| init.clone());
            Some(operation(acc, key, val))
        })
    }

    pub fn fold_first<FO>(self, mut operation: FO) -> HashMap<K, V>
    where
        FO: FnMut(V, &K, V) -> V,
    {
        self.aggregate(|acc, key, val| {
            Some(match acc {
                Some(acc) => operation(acc, key, val),
                None => val,
            })
        })
    }

    pub fn collect<C>(self) -> HashMap<K, C>
    where
        C: Default + Extend<V>,
    {
        self.aggregate(|acc, _, v| {
            let mut acc = acc.unwrap_or_else(C::default);
            acc.extend(Some(v));
            Some(acc)
        })
    }

    pub fn count(self) -> HashMap<K, usize> {
        self.fold(0, |acc, _, _| acc + 1)
    }
}
