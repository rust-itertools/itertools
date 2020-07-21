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

/// `GroupingMap` is an intermediate struct for efficient group-and-fold operations.
/// It groups elements by their key and at the same time fold each group
/// using some aggregating operation.
/// 
/// No method on this struct performs temporary allocations.
pub struct GroupingMap<I> {
    iter: I,
}

impl<I, K, V> GroupingMap<I>
where
    I: Iterator<Item = (K, V)>,
    K: Hash + Eq,
{
    /// This is the generic way to perform any operations on a `GroupingMap`.
    /// It's suggested to use this method only to implement custom operations
    /// when the already provided ones are not enough.
    /// 
    /// Groups elements from the `GroupingMap` source by key and applies `operation` to the elements
    /// of each group sequentially, passing the previously accumulated value, a reference to the key
    /// and the current element as arguments, and stores the results in an `HashMap`.
    ///
    /// The `operation` function is invoked on each element with the following parameters:
    ///  - the current value of the accumulator of the group if there is currently one;
    ///  - a reference to the key of the group this element belongs to;
    ///  - the element from the source being aggregated;
    /// If `operation` returns `Some(element)` then the accumulator is updated with `element`,
    /// otherwise the previous accumulation is discarded.
    ///
    /// Return a `HashMap` associating the key of each group with the result of aggregation of the group elements.
    /// If there's no result then there won't be an entry associated to that key.
    /// 
    /// ```
    /// use itertools::Itertools;
    /// 
    /// let data = vec![10, 5, 7, 9, 0, 4, 2];
    /// let lookup = data.into_iter()
    ///     .map(|n| (n % 4, n))
    ///     .into_grouping_map()
    ///     .aggregate(|acc, _, val| {
    ///         match val {
    ///             0 | 2 => None,
    ///             _ => Some(acc.unwrap_or(0) + val)
    ///         }
    ///     });
    /// 
    /// assert_eq!(lookup[&0], 4);
    /// assert_eq!(lookup[&1], 14);
    /// assert!(!lookup.contains_key(&2));
    /// assert_eq!(lookup[&3], 7);
    /// assert_eq!(lookup.len(), 3);
    /// ```
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

    /// Groups elements from the `GroupingMap` source by key and applies `operation` to the elements
    /// of each group sequentially, passing the previously accumulated value, a reference to the key
    /// and the current element as arguments, and stores the results in a new map.
    ///
    /// `init` is the value from which will be cloned the initial value of each accumulator.
    ///
    /// `operation` is a function that is invoked on each element with the following parameters:
    ///  - the current value of the accumulator of the group;
    ///  - a reference to the key of the group this element belongs to;
    ///  - the element from the source being accumulated.
    ///
    /// Return a `HashMap` associating the key of each group with the result of folding the group elements.
    /// 
    /// ```
    /// use itertools::Itertools;
    /// 
    /// let lookup = (1..=7)
    ///     .map(|n| (n % 3, n))
    ///     .into_grouping_map()
    ///     .fold(0, |acc, _, val| acc + val);
    /// 
    /// assert_eq!(lookup[&0], 9);
    /// assert_eq!(lookup[&1], 12);
    /// assert_eq!(lookup[&2], 7);
    /// assert_eq!(lookup.len(), 3);
    /// ```
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

    /// Groups elements from the `GroupingMap` source by key and applies `operation` to the elements
    /// of each group sequentially, passing the previously accumulated value, a reference to the key
    /// and the current element as arguments, and stores the results in a new map.
    ///
    /// This is similar to [`fold`] but the initial value of the accumulator is the first element of the group.
    ///
    /// `operation` is a function that is invoked on each element with the following parameters:
    ///  - the current value of the accumulator of the group;
    ///  - a reference to the key of the group this element belongs to;
    ///  - the element from the source being accumulated.
    ///
    /// Return a `HashMap` associating the key of each group with the result of folding the group elements.
    /// 
    /// [`fold`]: #tymethod.fold
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

    /// Groups elements from the `GroupingMap` source by key and collects the elements of each group in
    /// an instance of `C`. The iteration order is preserved when inserting elements. 
    /// 
    /// Return a `HashMap` associating the key of each group with the collection containing the elements of that group.
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

    /// Groups elements from the `GroupingMap` source by key and counts them.
    /// 
    /// Return a `HashMap` associating the key of each group with the number of elements in that group.
    pub fn count(self) -> HashMap<K, usize> {
        self.fold(0, |acc, _, _| acc + 1)
    }
}
