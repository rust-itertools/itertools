//! SQL-like join implementation of two (non-sorted) iterators.
//!
//! The hash join strategy requires the right iterator can be collected to a `HashMap`. The left
//! iterator can be arbitrarily long. It is therefore asymmetric (iterators cannot be swapped), as
//! distinct from [the merge join strategy](merge_join/index.html), which is symmetric.
//!
//! The fact that iterators do not need to be sorted makes it very efficient and particularly
//! suitable for [star schema](https://en.wikipedia.org/wiki/Star_schema) or [snowflake
//! schema](https://en.wikipedia.org/wiki/Snowflake_schema) joins.
//!
//! The supported join types:
//!
//! * [`INNER JOIN`](trait.Itertools.html#method.hash_join_inner) - an intersection between the
//! left and the right iterator.
//! * [`LEFT EXCL JOIN`](trait.Itertools.html#method.hash_join_left_excl) - a difference
//! between the left and the right iterator (not directly in SQL).
//! * [`LEFT OUTER JOIN`](trait.Itertools.html#method.hash_join_left_outer) - a union of `INNER
//! JOIN` and `LEFT EXCL JOIN`.
//! * [`RIGHT EXCL JOIN`](trait.Itertools.html#method.hash_join_right_excl) - a difference
//! between the right and the left iterator (not directly in SQL).
//! * [`RIGHT OUTER JOIN`](trait.Itertools.html#method.hash_join_right_outer) - a union of `INNER
//! JOIN` and `RIGHT EXCL JOIN`.
//! * [`FULL OUTER JOIN`](trait.Itertools.html#method.hash_join_full_outer) - a union of `INNER
//! JOIN`, `LEFT EXCL JOIN` and `RIGHT EXCL JOIN`.

use std::collections::hash_map::{HashMap, IntoIter,};
use std::mem;
use std::hash::Hash;
use super::EitherOrBoth::{self, Right, Left, Both};

/// An iterator adaptor that [inner joins](https://en.wikipedia.org/wiki/Join_%28SQL%29#Inner_join)
/// the two base iterators in ascending order. The resulting iterator is the intersection of the
/// two base iterators.
///
/// The base iterators do *not* need to be sorted. The right base iterator is loaded into
/// `HashMap` and thus must be unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by), if necessary) to produce the correct
/// results. The left base iterator do not need to be unique on the key.
///
/// The left base iterator element type must be `(K, LV)`, where `K: Hash + Eq`. 
/// The right base iterator element type must be `(K, RV)`, where `K: Hash + Eq` and `RV:
/// Clone`.
///
/// When the join adaptor is created, the right iterator is **consumed** into `HashMap`.
///
/// Iterator element type is `(LV, RV)`. 
/// The `RV` is cloned from `HashMap` for each joined value. It is expected a single `RV` will
/// be joined (and cloned) multiple times to `LV`. To increase performance, consider wrapping
/// `RV` into `std::rc::Rc` pointer to avoid unnecessary allocations.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HashJoinInner<L, K, RV> {
    left: L,
    map: HashMap<K, RV>,
}

impl<L, K, RV> HashJoinInner<L, K, RV> 
    where K: Hash + Eq,
{
    /// Create a `HashJoinInner` iterator.
    pub fn new<LI, RI>(left: LI, right: RI) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              RI: IntoIterator<Item=(K, RV)>
    {
        let mut map: HashMap<K, RV> = HashMap::new();
        for (k, v) in right.into_iter() {
            map.insert(k, v);
        }
        HashJoinInner {
            left: left.into_iter(),
            map: map,
        }
    }
}

impl<L, K, LV, RV> Iterator for HashJoinInner<L, K, RV> 
    where L: Iterator<Item=(K, LV)>,
          K: Hash + Eq,
          RV: Clone,
{
    type Item = (LV, RV);
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.left.next() {
                Some((lk, lv)) => match self.map.get(&lk) {
                    Some(rv) => return Some((lv, rv.clone())),
                    None => continue,
                },
                None => return None,
            }
        }
    }
}

/// An iterator adaptor that *left exclusive joins* the two base iterators. The resulting iterator
/// contains only those records from the left input iterator, which do not match the right input
/// iterator. There is no direct equivalent in SQL.
///
/// The base iterators do *not* need to be sorted. The right base iterator is loaded into
/// `HashMap` and thus must be unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by), if necessary) to produce the correct
/// results. The left base iterator do not need to be unique on the key.
///
/// The left base iterator element type must be `(K, LV)`, where `K: Hash + Eq`. 
/// The right base iterator element type must be `(K, RV)`, where `K: Hash + Eq`.
///
/// When the join adaptor is created, the right iterator is **consumed** into `HashMap`.
///
/// Iterator element type is `LV`.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HashJoinLeftExcl<L, K, RV> {
    left: L,
    map: HashMap<K, RV>,
}

impl<L, K, RV> HashJoinLeftExcl<L, K, RV> 
    where K: Hash + Eq,
{
    /// Create a `HashJoinLeftExcl` iterator.
    pub fn new<LI, RI>(left: LI, right: RI) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              RI: IntoIterator<Item=(K, RV)>
    {
        let mut map: HashMap<K, RV> = HashMap::new();
        for (k, v) in right.into_iter() {
            map.insert(k, v);
        }
        HashJoinLeftExcl {
            left: left.into_iter(),
            map: map,
        }
    }
}

impl<L, K, LV, RV> Iterator for HashJoinLeftExcl<L, K, RV> 
    where L: Iterator<Item=(K, LV)>,
          K: Hash + Eq,
{
    type Item = LV;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.left.next() {
                Some((lk, lv)) => match self.map.get(&lk) {
                    Some(_) => continue,
                    None => return Some(lv),
                },
                None => return None,
            }
        }
    }
}

/// An iterator adaptor that [left outer
/// joins](https://en.wikipedia.org/wiki/Join_%28SQL%29#Left_outer_join) the two base iterators.
/// The resulting iterator contains all the records from the left input iterator, even if they do
/// not match the right input iterator.
///
/// The base iterators do *not* need to be sorted. The right base iterator is loaded into
/// `HashMap` and thus must be unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by), if necessary) to produce the correct
/// results. The left base iterator do not need to be unique on the key.
///
/// The left base iterator element type must be `(K, LV)`, where `K: Hash + Eq`. 
/// The right base iterator element type must be `(K, RV)`, where `K: Hash + Eq` and `RV:
/// Clone`.
///
/// When the join adaptor is created, the right iterator is **consumed** into `HashMap`.
///
/// Iterator element type is [`EitherOrBoth<LV, RV>`](enum.EitherOrBoth.html).
/// The `RV` is cloned from `HashMap` for each joined value. It is expected a single `RV` will
/// be joined (and cloned) multiple times to `LV`. To increase performance, consider wrapping
/// `RV` into `std::rc::Rc` pointer to avoid unnecessary allocations.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HashJoinLeftOuter<L, K, RV> {
    left: L,
    map: HashMap<K, RV>,
}

impl<L, K, RV> HashJoinLeftOuter<L, K, RV> 
    where K: Hash + Eq,
{
    /// Create a `HashJoinLeftOuter` iterator.
    pub fn new<LI, RI>(left: LI, right: RI) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              RI: IntoIterator<Item=(K, RV)>
    {
        let mut map: HashMap<K, RV> = HashMap::new();
        for (k, v) in right.into_iter() {
            map.insert(k, v);
        }
        HashJoinLeftOuter {
            left: left.into_iter(),
            map: map,
        }
    }
}

impl<L, K, LV, RV> Iterator for HashJoinLeftOuter<L, K, RV> 
    where L: Iterator<Item=(K, LV)>,
          K: Hash + Eq,
          RV: Clone,
{
    type Item = EitherOrBoth<LV, RV>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.left.next() {
                Some((lk, lv)) => match self.map.get(&lk) {
                    Some(rv) => return Some(Both(lv, rv.clone())),
                    None => return Some(Left(lv)),
                },
                None => return None,
            }
        }
    }
}

/// An iterator adaptor that *right exclusive joins* the two base iterators. The resulting iterator
/// contains only those records from the right input iterator, which do not match the left input
/// iterator. There is no direct equivalent in SQL.
///
/// The base iterators do *not* need to be sorted. The right base iterator is loaded into
/// `HashMap` and thus must be unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by), if necessary) to produce the correct
/// results. The left base iterator do not need to be unique on the key.
///
/// The left base iterator element type must be `(K, LV)`, where `K: Hash + Eq`. 
/// The right base iterator element type must be `(K, RV)`, where `K: Hash + Eq`.
///
/// When the join adaptor is created, the right iterator is **consumed** into `HashMap`.
///
/// Iterator element type is `RV`.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HashJoinRightExcl<L, K, RV> {
    left: L,
    map: HashMap<K, (RV, bool)>,
    /// exclusion iterator - yields the unmatched values from the map. It is created once the left
    /// iterator is exhausted
    excl_iter: Option<IntoIter<K, (RV, bool)>>,
}

impl<L, K, RV> HashJoinRightExcl<L, K, RV> 
    where K: Hash + Eq,
{
    /// Create a `HashJoinRightExcl` iterator.
    pub fn new<LI, RI>(left: LI, right: RI) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              RI: IntoIterator<Item=(K, RV)>
    {
        let mut map: HashMap<K, (RV, bool)> = HashMap::new();
        for (k, v) in right.into_iter() {
            map.insert(k, (v, false));
        }
        HashJoinRightExcl {
            left: left.into_iter(),
            map: map,
            excl_iter: None,
        }
    }

    /// Moves the map to `self.excl_iter`
    ///
    /// Once the left iterator is exhausted, the info about which keys were matched is complete.
    /// To be able to iterate over map's values we need to move it into its `IntoIter`.
    fn set_excl_iter(&mut self) {
        let map = mem::replace(&mut self.map, HashMap::<K, (RV, bool)>::new());
        self.excl_iter = Some(map.into_iter());
    }
}

impl<L, K, LV, RV> Iterator for HashJoinRightExcl<L, K, RV> 
    where L: Iterator<Item=(K, LV)>,
          K: Hash + Eq,
{
    type Item = RV;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.excl_iter {
                // the left iterator is not yet exhausted
                None => match self.left.next() {
                    Some((lk, _)) => match self.map.get_mut(&lk) {
                        Some(rt) => {
                            rt.1 = true; // flag as matched
                        },
                        None => continue, // not interested in unmatched left value
                    },
                    // the left iterator is exhausted so move the map into `self.excl_iter`.
                    None => self.set_excl_iter(),
                },
                // iterate over unmatched values
                Some(ref mut r) => match r.next() {
                    Some((_, (rv, matched))) => {
                        if !matched {
                            return Some(rv);
                        } else {
                            continue;
                        }
                    },
                    None => return None,
                }
            }
        }
    }
}

/// An iterator adaptor that [right outer
/// joins](https://en.wikipedia.org/wiki/Join_%28SQL%29#Right_outer_join) the two base iterators.
/// The resulting iterator contains all the records from the right input iterator, even if they do
/// not match the left input iterator.
///
/// The base iterators do *not* need to be sorted. The right base iterator is loaded into
/// `HashMap` and thus must be unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by), if necessary) to produce the correct
/// results. The left base iterator do not need to be unique on the key.
///
/// The left base iterator element type must be `(K, LV)`, where `K: Hash + Eq`. 
/// The right base iterator element type must be `(K, RV)`, where `K: Hash + Eq` and `RV:
/// Clone`.
///
/// When the join adaptor is created, the right iterator is **consumed** into `HashMap`.
///
/// Iterator element type is [`EitherOrBoth<LV, RV>`](enum.EitherOrBoth.html).
/// The `RV` is cloned from `HashMap` for each joined value. It is expected a single `RV` will
/// be joined (and cloned) multiple times to `LV`. To increase performance, consider wrapping
/// `RV` into `std::rc::Rc` pointer to avoid unnecessary allocations.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HashJoinRightOuter<L, K, RV> {
    left: L,
    map: HashMap<K, (RV, bool)>,
    /// exclusion iterator - yields the unmatched values from the map. It is created once the left
    /// iterator is exhausted
    excl_iter: Option<IntoIter<K, (RV, bool)>>,
}

impl<L, K, RV> HashJoinRightOuter<L, K, RV> 
    where K: Hash + Eq,
{
    /// Create a `HashJoinRightOuter` iterator.
    pub fn new<LI, RI>(left: LI, right: RI) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              RI: IntoIterator<Item=(K, RV)>
    {
        let mut map: HashMap<K, (RV, bool)> = HashMap::new();
        for (k, v) in right.into_iter() {
            map.insert(k, (v, false));
        }
        HashJoinRightOuter {
            left: left.into_iter(),
            map: map,
            excl_iter: None,
        }
    }

    /// Moves the map to `self.excl_iter`
    ///
    /// Once the left iterator is exhausted, the info about which keys were matched is complete.
    /// To be able to iterate over map's values we need to move it into its `IntoIter`.
    fn set_excl_iter(&mut self) {
        let map = mem::replace(&mut self.map, HashMap::<K, (RV, bool)>::new());
        self.excl_iter = Some(map.into_iter());
    }
}

impl<L, K, LV, RV> Iterator for HashJoinRightOuter<L, K, RV> 
    where L: Iterator<Item=(K, LV)>,
          K: Hash + Eq,
          RV: Clone,
{
    type Item = EitherOrBoth<LV, RV>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.excl_iter {
                // the left iterator is not yet exhausted
                None => match self.left.next() {
                    Some((lk, lv)) => match self.map.get_mut(&lk) {
                        Some(rt) => {
                            rt.1 = true; // flag as matched
                            return Some(Both(lv, rt.0.clone()))
                        },
                        None => continue, // not interested in unmatched left value
                    },
                    // the left iterator is exhausted so move the map into `self.excl_iter`.
                    None => self.set_excl_iter(),
                },
                // iterate over unmatched values
                Some(ref mut r) => match r.next() {
                    Some((_, (rv, matched))) => {
                        if !matched {
                            return Some(Right(rv));
                        } else {
                            continue;
                        }
                    },
                    None => return None,
                }
            }
        }
    }
}

/// An iterator adaptor that [full outer
/// joins](https://en.wikipedia.org/wiki/Join_%28SQL%29#Full_outer_join) the two base iterators.
/// The resulting iterator contains all the records from the both input iterators.
///
/// The base iterators do *not* need to be sorted. The right base iterator is loaded into
/// `HashMap` and thus must be unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by), if necessary) to produce the correct
/// results. The left base iterator do not need to be unique on the key.
///
/// The left base iterator element type must be `(K, LV)`, where `K: Hash + Eq`. 
/// The right base iterator element type must be `(K, RV)`, where `K: Hash + Eq` and `RV:
/// Clone`.
///
/// When the join adaptor is created, the right iterator is **consumed** into `HashMap`.
///
/// Iterator element type is [`EitherOrBoth<LV, RV>`](enum.EitherOrBoth.html).
/// The `RV` is cloned from `HashMap` for each joined value. It is expected a single `RV` will
/// be joined (and cloned) multiple times to `LV`. To increase performance, consider wrapping
/// `RV` into `std::rc::Rc` pointer to avoid unnecessary allocations.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HashJoinFullOuter<L, K, RV> {
    left: L,
    map: HashMap<K, (RV, bool)>,
    /// exclusion iterator - yields the unmatched values from the map. It is created once the left
    /// iterator is exhausted
    excl_iter: Option<IntoIter<K, (RV, bool)>>,
}

impl<L, K, RV> HashJoinFullOuter<L, K, RV> 
    where K: Hash + Eq,
{
    /// Create a `HashJoinFullOuter` iterator.
    pub fn new<LI, RI>(left: LI, right: RI) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              RI: IntoIterator<Item=(K, RV)>
    {
        let mut map: HashMap<K, (RV, bool)> = HashMap::new();
        for (k, v) in right.into_iter() {
            map.insert(k, (v, false));
        }
        HashJoinFullOuter {
            left: left.into_iter(),
            map: map,
            excl_iter: None,
        }
    }

    /// Moves the map to `self.excl_iter`
    ///
    /// Once the left iterator is exhausted, the info about which keys were matched is complete.
    /// To be able to iterate over map's values we need to move it into its `IntoIter`.
    fn set_excl_iter(&mut self) {
        let map = mem::replace(&mut self.map, HashMap::<K, (RV, bool)>::new());
        self.excl_iter = Some(map.into_iter());
    }
}

impl<L, K, LV, RV> Iterator for HashJoinFullOuter<L, K, RV> 
    where L: Iterator<Item=(K, LV)>,
          K: Hash + Eq,
          RV: Clone,
{
    type Item = EitherOrBoth<LV, RV>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.excl_iter {
                // the left iterator is not yet exhausted
                None => match self.left.next() {
                    Some((lk, lv)) => match self.map.get_mut(&lk) {
                        Some(rt) => {
                            rt.1 = true; // flag as matched
                            return Some(Both(lv, rt.0.clone()))
                        },
                        None => return Some(Left(lv)),
                    },
                    // the left iterator is exhausted so move the map into `self.excl_iter`.
                    None => self.set_excl_iter(),
                },
                // iterate over unmatched values
                Some(ref mut r) => match r.next() {
                    Some((_, (rv, matched))) => {
                        if !matched {
                            return Some(Right(rv));
                        } else {
                            continue;
                        }
                    },
                    None => return None,
                }
            }
        }
    }
}
