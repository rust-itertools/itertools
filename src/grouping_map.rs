#![cfg(feature = "use_alloc")]

use crate::{
    adaptors::map::{MapSpecialCase, MapSpecialCaseFn},
    generic_containers::Map,
    MinMaxResult,
};
use std::cmp::Ordering;
use std::iter::Iterator;
use std::ops::{Add, Mul};

#[cfg(feature = "use_std")]
pub use with_hashmap::{GroupingMap, GroupingMapBy};

#[cfg(feature = "use_std")]
mod with_hashmap {
    use super::*;
    use std::collections::HashMap;

    // This is used to infer `K` when `I::Item = (K, V)` since we can't write `I::Item.0`.
    pub trait KeyValue {
        type Key;
    }

    impl<K, V> KeyValue for (K, V) {
        type Key = K;
    }

    /// `GroupingMap` is an intermediate struct for efficient group-and-fold operations.
    ///
    /// See [`GroupingGenericMap`] for more informations.
    pub type GroupingMap<I, R> =
        GroupingGenericMap<I, HashMap<<<I as Iterator>::Item as KeyValue>::Key, R>>;

    /// `GroupingMapBy` is an intermediate struct for efficient group-and-fold operations.
    ///
    /// See [`GroupingGenericMap`] for more informations.
    pub type GroupingMapBy<I, F, R> = GroupingMap<MapForGrouping<I, F>, R>;
}

/// A wrapper to allow for an easy [`into_grouping_map_by`](crate::Itertools::into_grouping_map_by)
pub type MapForGrouping<I, F> = MapSpecialCase<I, GroupingMapFn<F>>;

#[derive(Clone)]
pub struct GroupingMapFn<F>(F);

impl<F> std::fmt::Debug for GroupingMapFn<F> {
    debug_fmt_fields!(GroupingMapFn,);
}

impl<V, K, F: FnMut(&V) -> K> MapSpecialCaseFn<V> for GroupingMapFn<F> {
    type Out = (K, V);
    fn call(&mut self, v: V) -> Self::Out {
        ((self.0)(&v), v)
    }
}

pub(crate) fn new_map_for_grouping<K, I: Iterator, F: FnMut(&I::Item) -> K>(
    iter: I,
    key_mapper: F,
) -> MapForGrouping<I, F> {
    MapSpecialCase {
        iter,
        f: GroupingMapFn(key_mapper),
    }
}

pub fn new_in<I, K, V, M>(iter: I, map: M) -> GroupingGenericMap<I, M>
where
    I: Iterator<Item = (K, V)>,
    K: Eq,
    M: Map<Key = K>,
{
    GroupingGenericMap { iter, map }
}

/// `GroupingGenericMapBy` is an intermediate struct for efficient group-and-fold operations.
///
/// See [`GroupingGenericMap`] for more informations.
pub type GroupingGenericMapBy<I, F, M> = GroupingGenericMap<MapForGrouping<I, F>, M>;

/// `GroupingGenericMap` is an intermediate struct for efficient group-and-fold operations.
/// It groups elements by their key and at the same time fold each group
/// using some aggregating operation.
///
/// No method on this struct performs temporary allocations.
#[derive(Clone, Debug)]
#[must_use = "GroupingGenericMap is lazy and do nothing unless consumed"]
pub struct GroupingGenericMap<I, M> {
    iter: I,
    map: M,
}

impl<I, K, V, M> GroupingGenericMap<I, M>
where
    I: Iterator<Item = (K, V)>,
    K: Eq,
    M: Map<Key = K>,
{
    /// This is the generic way to perform any operation on a `GroupingGenericMap`.
    /// It's suggested to use this method only to implement custom operations
    /// when the already provided ones are not enough.
    ///
    /// Groups elements from the `GroupingGenericMap` source by key and applies `operation` to the elements
    /// of each group sequentially, passing the previously accumulated value, a reference to the key
    /// and the current element as arguments, and stores the results in an `HashMap`.
    ///
    /// The `operation` function is invoked on each element with the following parameters:
    ///  - the current value of the accumulator of the group if there is currently one;
    ///  - a reference to the key of the group this element belongs to;
    ///  - the element from the source being aggregated;
    ///
    /// If `operation` returns `Some(element)` then the accumulator is updated with `element`,
    /// otherwise the previous accumulation is discarded.
    ///
    /// Return a `HashMap` associating the key of each group with the result of aggregation of
    /// that group's elements. If the aggregation of the last element of a group discards the
    /// accumulator then there won't be an entry associated to that group's key.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let data = vec![2, 8, 5, 7, 9, 0, 4, 10];
    /// let lookup = data.into_iter()
    ///     .into_grouping_map_by(|&n| n % 4)
    ///     .aggregate(|acc, _key, val| {
    ///         if val == 0 || val == 10 {
    ///             None
    ///         } else {
    ///             Some(acc.unwrap_or(0) + val)
    ///         }
    ///     });
    ///
    /// assert_eq!(lookup[&0], 4);        // 0 resets the accumulator so only 4 is summed
    /// assert_eq!(lookup[&1], 5 + 9);
    /// assert_eq!(lookup.get(&2), None); // 10 resets the accumulator and nothing is summed afterward
    /// assert_eq!(lookup[&3], 7);
    /// assert_eq!(lookup.len(), 3);      // The final keys are only 0, 1 and 2
    /// ```
    pub fn aggregate<FO, R>(self, mut operation: FO) -> M
    where
        FO: FnMut(Option<R>, &K, V) -> Option<R>,
        M: Map<Value = R>,
    {
        let mut destination_map = self.map;

        self.iter
            .for_each(|(key, val)| destination_map.aggregate(key, val, &mut operation));

        destination_map
    }

    /// Groups elements from the `GroupingGenericMap` source by key and applies `operation` to the elements
    /// of each group sequentially, passing the previously accumulated value, a reference to the key
    /// and the current element as arguments, and stores the results in a new map.
    ///
    /// `init` is called to obtain the initial value of each accumulator.
    ///
    /// `operation` is a function that is invoked on each element with the following parameters:
    ///  - the current value of the accumulator of the group;
    ///  - a reference to the key of the group this element belongs to;
    ///  - the element from the source being accumulated.
    ///
    /// Return a `HashMap` associating the key of each group with the result of folding that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// #[derive(Debug, Default)]
    /// struct Accumulator {
    ///   acc: usize,
    /// }
    ///
    /// let lookup = (1..=7)
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .fold_with(|_key, _val| Default::default(), |Accumulator { acc }, _key, val| {
    ///         let acc = acc + val;
    ///         Accumulator { acc }
    ///      });
    ///
    /// assert_eq!(lookup[&0].acc, 3 + 6);
    /// assert_eq!(lookup[&1].acc, 1 + 4 + 7);
    /// assert_eq!(lookup[&2].acc, 2 + 5);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn fold_with<FI, FO, R>(self, mut init: FI, mut operation: FO) -> M
    where
        FI: FnMut(&K, &V) -> R,
        FO: FnMut(R, &K, V) -> R,
        M: Map<Value = R>,
    {
        self.aggregate(|acc, key, val| {
            let acc = acc.unwrap_or_else(|| init(key, &val));
            Some(operation(acc, key, val))
        })
    }

    /// Groups elements from the `GroupingGenericMap` source by key and applies `operation` to the elements
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
    /// Return a `HashMap` associating the key of each group with the result of folding that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = (1..=7)
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .fold(0, |acc, _key, val| acc + val);
    ///
    /// assert_eq!(lookup[&0], 3 + 6);
    /// assert_eq!(lookup[&1], 1 + 4 + 7);
    /// assert_eq!(lookup[&2], 2 + 5);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn fold<FO, R>(self, init: R, operation: FO) -> M
    where
        R: Clone,
        FO: FnMut(R, &K, V) -> R,
        M: Map<Value = R>,
    {
        self.fold_with(|_, _| init.clone(), operation)
    }

    /// Groups elements from the `GroupingGenericMap` source by key and applies `operation` to the elements
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
    /// Return a `HashMap` associating the key of each group with the result of folding that group's elements.
    ///
    /// [`fold`]: GroupingGenericMap::fold
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = (1..=7)
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .reduce(|acc, _key, val| acc + val);
    ///
    /// assert_eq!(lookup[&0], 3 + 6);
    /// assert_eq!(lookup[&1], 1 + 4 + 7);
    /// assert_eq!(lookup[&2], 2 + 5);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn reduce<FO>(self, mut operation: FO) -> M
    where
        FO: FnMut(V, &K, V) -> V,
        M: Map<Value = V>,
    {
        self.aggregate(|acc, key, val| {
            Some(match acc {
                Some(acc) => operation(acc, key, val),
                None => val,
            })
        })
    }

    /// See [`.reduce()`](GroupingGenericMap::reduce).
    #[deprecated(note = "Use .reduce() instead", since = "0.13.0")]
    pub fn fold_first<FO>(self, operation: FO) -> M
    where
        FO: FnMut(V, &K, V) -> V,
        M: Map<Value = V>,
    {
        self.reduce(operation)
    }

    /// Groups elements from the `GroupingGenericMap` source by key and collects the elements of each group in
    /// an instance of `C`. The iteration order is preserved when inserting elements.
    ///
    /// Return a `HashMap` associating the key of each group with the collection containing that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    /// use std::collections::HashSet;
    ///
    /// let lookup = vec![0, 1, 2, 3, 4, 5, 6, 2, 3, 6].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .collect::<HashSet<_>>();
    ///
    /// assert_eq!(lookup[&0], vec![0, 3, 6].into_iter().collect::<HashSet<_>>());
    /// assert_eq!(lookup[&1], vec![1, 4].into_iter().collect::<HashSet<_>>());
    /// assert_eq!(lookup[&2], vec![2, 5].into_iter().collect::<HashSet<_>>());
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn collect<C>(self) -> M
    where
        C: Default + Extend<V>,
        M: Map<Value = C>,
    {
        let mut destination_map = self.map;

        self.iter.for_each(|(key, val)| {
            destination_map.entry_or_default(key).extend(Some(val));
        });

        destination_map
    }

    /// Groups elements from the `GroupingGenericMap` source by key and finds the maximum of each group.
    ///
    /// If several elements are equally maximum, the last element is picked.
    ///
    /// Returns a `HashMap` associating the key of each group with the maximum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .max();
    ///
    /// assert_eq!(lookup[&0], 12);
    /// assert_eq!(lookup[&1], 7);
    /// assert_eq!(lookup[&2], 8);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn max(self) -> M
    where
        V: Ord,
        M: Map<Value = V>,
    {
        self.max_by(|_, v1, v2| V::cmp(v1, v2))
    }

    /// Groups elements from the `GroupingGenericMap` source by key and finds the maximum of each group
    /// with respect to the specified comparison function.
    ///
    /// If several elements are equally maximum, the last element is picked.
    ///
    /// Returns a `HashMap` associating the key of each group with the maximum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .max_by(|_key, x, y| y.cmp(x));
    ///
    /// assert_eq!(lookup[&0], 3);
    /// assert_eq!(lookup[&1], 1);
    /// assert_eq!(lookup[&2], 5);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn max_by<F>(self, mut compare: F) -> M
    where
        F: FnMut(&K, &V, &V) -> Ordering,
        M: Map<Value = V>,
    {
        self.reduce(|acc, key, val| match compare(key, &acc, &val) {
            Ordering::Less | Ordering::Equal => val,
            Ordering::Greater => acc,
        })
    }

    /// Groups elements from the `GroupingGenericMap` source by key and finds the element of each group
    /// that gives the maximum from the specified function.
    ///
    /// If several elements are equally maximum, the last element is picked.
    ///
    /// Returns a `HashMap` associating the key of each group with the maximum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .max_by_key(|_key, &val| val % 4);
    ///
    /// assert_eq!(lookup[&0], 3);
    /// assert_eq!(lookup[&1], 7);
    /// assert_eq!(lookup[&2], 5);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn max_by_key<F, CK>(self, mut f: F) -> M
    where
        F: FnMut(&K, &V) -> CK,
        CK: Ord,
        M: Map<Value = V>,
    {
        self.max_by(|key, v1, v2| f(key, v1).cmp(&f(key, v2)))
    }

    /// Groups elements from the `GroupingGenericMap` source by key and finds the minimum of each group.
    ///
    /// If several elements are equally minimum, the first element is picked.
    ///
    /// Returns a `HashMap` associating the key of each group with the minimum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .min();
    ///
    /// assert_eq!(lookup[&0], 3);
    /// assert_eq!(lookup[&1], 1);
    /// assert_eq!(lookup[&2], 5);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn min(self) -> M
    where
        V: Ord,
        M: Map<Value = V>,
    {
        self.min_by(|_, v1, v2| V::cmp(v1, v2))
    }

    /// Groups elements from the `GroupingGenericMap` source by key and finds the minimum of each group
    /// with respect to the specified comparison function.
    ///
    /// If several elements are equally minimum, the first element is picked.
    ///
    /// Returns a `HashMap` associating the key of each group with the minimum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .min_by(|_key, x, y| y.cmp(x));
    ///
    /// assert_eq!(lookup[&0], 12);
    /// assert_eq!(lookup[&1], 7);
    /// assert_eq!(lookup[&2], 8);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn min_by<F>(self, mut compare: F) -> M
    where
        F: FnMut(&K, &V, &V) -> Ordering,
        M: Map<Value = V>,
    {
        self.reduce(|acc, key, val| match compare(key, &acc, &val) {
            Ordering::Less | Ordering::Equal => acc,
            Ordering::Greater => val,
        })
    }

    /// Groups elements from the `GroupingGenericMap` source by key and finds the element of each group
    /// that gives the minimum from the specified function.
    ///
    /// If several elements are equally minimum, the first element is picked.
    ///
    /// Returns a `HashMap` associating the key of each group with the minimum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .min_by_key(|_key, &val| val % 4);
    ///
    /// assert_eq!(lookup[&0], 12);
    /// assert_eq!(lookup[&1], 4);
    /// assert_eq!(lookup[&2], 8);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn min_by_key<F, CK>(self, mut f: F) -> M
    where
        F: FnMut(&K, &V) -> CK,
        CK: Ord,
        M: Map<Value = V>,
    {
        self.min_by(|key, v1, v2| f(key, v1).cmp(&f(key, v2)))
    }

    /// Groups elements from the `GroupingGenericMap` source by key and find the maximum and minimum of
    /// each group.
    ///
    /// If several elements are equally maximum, the last element is picked.
    /// If several elements are equally minimum, the first element is picked.
    ///
    /// See [.minmax()](crate::Itertools::minmax) for the non-grouping version.
    ///
    /// Differences from the non grouping version:
    /// - It never produces a `MinMaxResult::NoElements`
    /// - It doesn't have any speedup
    ///
    /// Returns a `HashMap` associating the key of each group with the minimum and maximum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    /// use itertools::MinMaxResult::{OneElement, MinMax};
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .minmax();
    ///
    /// assert_eq!(lookup[&0], MinMax(3, 12));
    /// assert_eq!(lookup[&1], MinMax(1, 7));
    /// assert_eq!(lookup[&2], OneElement(5));
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn minmax(self) -> M
    where
        V: Ord,
        M: Map<Value = MinMaxResult<V>>,
    {
        self.minmax_by(|_, v1, v2| V::cmp(v1, v2))
    }

    /// Groups elements from the `GroupingGenericMap` source by key and find the maximum and minimum of
    /// each group with respect to the specified comparison function.
    ///
    /// If several elements are equally maximum, the last element is picked.
    /// If several elements are equally minimum, the first element is picked.
    ///
    /// It has the same differences from the non-grouping version as `minmax`.
    ///
    /// Returns a `HashMap` associating the key of each group with the minimum and maximum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    /// use itertools::MinMaxResult::{OneElement, MinMax};
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .minmax_by(|_key, x, y| y.cmp(x));
    ///
    /// assert_eq!(lookup[&0], MinMax(12, 3));
    /// assert_eq!(lookup[&1], MinMax(7, 1));
    /// assert_eq!(lookup[&2], OneElement(5));
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn minmax_by<F>(self, mut compare: F) -> M
    where
        F: FnMut(&K, &V, &V) -> Ordering,
        M: Map<Value = MinMaxResult<V>>,
    {
        self.aggregate(|acc, key, val| {
            Some(match acc {
                Some(MinMaxResult::OneElement(e)) => {
                    if compare(key, &val, &e) == Ordering::Less {
                        MinMaxResult::MinMax(val, e)
                    } else {
                        MinMaxResult::MinMax(e, val)
                    }
                }
                Some(MinMaxResult::MinMax(min, max)) => {
                    if compare(key, &val, &min) == Ordering::Less {
                        MinMaxResult::MinMax(val, max)
                    } else if compare(key, &val, &max) != Ordering::Less {
                        MinMaxResult::MinMax(min, val)
                    } else {
                        MinMaxResult::MinMax(min, max)
                    }
                }
                None => MinMaxResult::OneElement(val),
                Some(MinMaxResult::NoElements) => unreachable!(),
            })
        })
    }

    /// Groups elements from the `GroupingGenericMap` source by key and find the elements of each group
    /// that gives the minimum and maximum from the specified function.
    ///
    /// If several elements are equally maximum, the last element is picked.
    /// If several elements are equally minimum, the first element is picked.
    ///
    /// It has the same differences from the non-grouping version as `minmax`.
    ///
    /// Returns a `HashMap` associating the key of each group with the minimum and maximum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    /// use itertools::MinMaxResult::{OneElement, MinMax};
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .minmax_by_key(|_key, &val| val % 4);
    ///
    /// assert_eq!(lookup[&0], MinMax(12, 3));
    /// assert_eq!(lookup[&1], MinMax(4, 7));
    /// assert_eq!(lookup[&2], OneElement(5));
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn minmax_by_key<F, CK>(self, mut f: F) -> M
    where
        F: FnMut(&K, &V) -> CK,
        CK: Ord,
        M: Map<Value = MinMaxResult<V>>,
    {
        self.minmax_by(|key, v1, v2| f(key, v1).cmp(&f(key, v2)))
    }

    /// Groups elements from the `GroupingGenericMap` source by key and sums them.
    ///
    /// This is just a shorthand for `self.reduce(|acc, _, val| acc + val)`.
    /// It is more limited than `Iterator::sum` since it doesn't use the `Sum` trait.
    ///
    /// Returns a `HashMap` associating the key of each group with the sum of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .sum();
    ///
    /// assert_eq!(lookup[&0], 3 + 9 + 12);
    /// assert_eq!(lookup[&1], 1 + 4 + 7);
    /// assert_eq!(lookup[&2], 5 + 8);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn sum(self) -> M
    where
        V: Add<V, Output = V>,
        M: Map<Value = V>,
    {
        self.reduce(|acc, _, val| acc + val)
    }

    /// Groups elements from the `GroupingGenericMap` source by key and multiply them.
    ///
    /// This is just a shorthand for `self.reduce(|acc, _, val| acc * val)`.
    /// It is more limited than `Iterator::product` since it doesn't use the `Product` trait.
    ///
    /// Returns a `HashMap` associating the key of each group with the product of that group's elements.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let lookup = vec![1, 3, 4, 5, 7, 8, 9, 12].into_iter()
    ///     .into_grouping_map_by(|&n| n % 3)
    ///     .product();
    ///
    /// assert_eq!(lookup[&0], 3 * 9 * 12);
    /// assert_eq!(lookup[&1], 1 * 4 * 7);
    /// assert_eq!(lookup[&2], 5 * 8);
    /// assert_eq!(lookup.len(), 3);
    /// ```
    pub fn product(self) -> M
    where
        V: Mul<V, Output = V>,
        M: Map<Value = V>,
    {
        self.reduce(|acc, _, val| acc * val)
    }
}
