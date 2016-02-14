//! SQL-like join implementation of two sorted iterators.
//!
//! The supported join types:
//!
//! * [`INNER JOIN`](trait.Itertools.html#method.merge_join_inner_by) - an intersection between the
//! left and the right iterator.
//! * [`LEFT EXCL JOIN`](trait.Itertools.html#method.merge_join_left_excl_by) - a difference
//! between the left and the right iterator (not directly in SQL).
//! * [`LEFT OUTER JOIN`](trait.Itertools.html#method.merge_join_left_outer_by) - a union of `INNER
//! JOIN` and `LEFT EXCL JOIN`.
//! * `RIGHT EXCL JOIN` - use the `LEFT EXCL JOIN` with left and right iterators swapped.
//! * `RIGHT OUTER JOIN` - use the `LEFT OUTER JOIN` with left and right iterators swapped.
//! * [`FULL OUTER JOIN`](trait.Itertools.html#method.merge_join_full_outer_by) - a union of `LEFT
//! EXCL JOIN` , `INNER JOIN` and `RIGHT EXCL JOIN`.
//!
//! A merge join strategy requires the two iterators to be sorted, which might be costly. Therefore,
//! if one of the iterators can be collected into memory, it is preferable to use a [hash
//! join](hash_join/index.html).
//!

use std::iter::{Peekable,};
use std::cmp::Ordering;
use super::EitherOrBoth::{self, Right, Left, Both};
 
/// An iterator adaptor that [inner
/// joins](https://en.wikipedia.org/wiki/Join_%28SQL%29#Inner_join) the two base iterators in
/// ascending order. The resulting iterator is the intersection of the two base iterators.
///
/// The both base iterators must be sorted (ascending) and unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by) them, if necessary) to produce the correct results.
///
/// Iterator element type is `(L::Item, R::Item)`.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MergeJoinInner<L, R, F> 
    where L: Iterator,
          R: Iterator,
{
    left: Peekable<L>,
    right: Peekable<R>,
    cmp: F,
}

impl<L, R, F> MergeJoinInner<L, R, F>
    where L: Iterator,
          R: Iterator,
{
    /// Create a `MergeJoinInner` iterator.
    pub fn new<LI, RI>(left: LI, right: RI, cmp: F) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              R: Iterator<Item=RI::Item>,
              RI: IntoIterator<IntoIter=R>,
              F: FnMut(&L::Item, &R::Item) -> Ordering
    {
        MergeJoinInner {
            left: left.into_iter().peekable(),
            right: right.into_iter().peekable(),
            cmp: cmp,
        }
    }
}

impl<L, R, F> Iterator for MergeJoinInner<L, R, F> 
    where L: Iterator,
          R: Iterator,
          F: FnMut(&L::Item, &R::Item) -> Ordering
{
    type Item = (L::Item, R::Item);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ord = match (self.left.peek(), self.right.peek()) {
                (Some(l), Some(r)) => (self.cmp)(l, r),
                _ => return None,
            };

            match ord {
                Ordering::Less => {self.left.next();},
                Ordering::Greater =>{self.right.next();},
                Ordering::Equal => match (self.left.next(), self.right.next()) {
                    (Some(l), Some(r)) => return Some((l, r)),
                    _ => return None,
                }
            }
        }
    }
}

/// An iterator adaptor that *left exclusive joins* the two base iterators in ascending order. The
/// resulting iterator contains only those records from the left input iterator, which do not match
/// the right input iterator. There is no direct equivalent in SQL.
/// 
/// The both base iterators must be sorted (ascending) and unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by) them, if necessary) to produce the correct results.
///
/// Iterator element type is `L::Item`.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MergeJoinLeftExcl<L, R, F> where
    L: Iterator,
    R: Iterator,
{
    left: Peekable<L>,
    right: Peekable<R>,
    cmp: F,
    fused: Option<Ordering>,
}

impl<L, R, F> MergeJoinLeftExcl<L, R, F> where
    L: Iterator,
    R: Iterator,
{
    /// Create a `MergeJoinLeftExcl` iterator.
    pub fn new<LI, RI>(left: LI, right: RI, cmp: F) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              R: Iterator<Item=RI::Item>,
              RI: IntoIterator<IntoIter=R>,
              F: FnMut(&L::Item, &R::Item) -> Ordering
    {
        MergeJoinLeftExcl {
            left: left.into_iter().peekable(),
            right: right.into_iter().peekable(),
            cmp: cmp,
            fused: None,
        }
    }
}

impl<L, R, F> Iterator for MergeJoinLeftExcl<L, R, F> 
    where L: Iterator,
          R: Iterator,
          F: FnMut(&L::Item, &R::Item) -> Ordering
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ord = match self.fused {
                Some(o) => o,
                None => match (self.left.peek(), self.right.peek()) {
                    (Some(l), Some(r)) => (self.cmp)(l, r),
                    (Some(_), None) => {
                        self.fused = Some(Ordering::Less);
                        Ordering::Less
                    }
                    _ => return None,
                }
            };

            match ord {
                Ordering::Less => return self.left.next(),
                Ordering::Greater => {self.right.next();},
                Ordering::Equal => {
                    self.left.next();
                    self.right.next();
                }
            }
        }
    }
}

/// An iterator adaptor that [left outer
/// joins](https://en.wikipedia.org/wiki/Join_%28SQL%29#Left_outer_join) the two base iterators in
/// ascending order. The resulting iterator contains all the records from the left input iterator,
/// even if they do not match the right input iterator.
///
/// The both base iterators must be sorted (ascending) and unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by) them, if necessary) to produce the correct results.
///
/// Iterator element type is [`EitherOrBoth<L::Item, R::Item>`](enum.EitherOrBoth.html).
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MergeJoinLeftOuter<L, R, F> where
    L: Iterator,
    R: Iterator,
{
    left: Peekable<L>,
    right: Peekable<R>,
    cmp: F,
    fused: Option<Ordering>,
}

impl<L, R, F> MergeJoinLeftOuter<L, R, F> where
    L: Iterator,
    R: Iterator,
{
    /// Create a `MergeJoinLeftOuter` iterator.
    pub fn new<LI, RI>(left: LI, right: RI, cmp: F) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              R: Iterator<Item=RI::Item>,
              RI: IntoIterator<IntoIter=R>,
              F: FnMut(&L::Item, &R::Item) -> Ordering
    {
        MergeJoinLeftOuter {
            left: left.into_iter().peekable(),
            right: right.into_iter().peekable(),
            cmp: cmp,
            fused: None,
        }
    }
}

impl<L, R, F> Iterator for MergeJoinLeftOuter<L, R, F>
    where L: Iterator,
          R: Iterator,
          F: FnMut(&L::Item, &R::Item) -> Ordering
{
    type Item = EitherOrBoth<L::Item, R::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ord = match self.fused {
                Some(o) => o,
                None => match (self.left.peek(), self.right.peek()) {
                    (Some(l), Some(r)) => (self.cmp)(l, r),
                    (Some(_), None) => {
                        self.fused = Some(Ordering::Less);
                        Ordering::Less
                    }
                    _ => return None,
                }
            };

            match ord {
                Ordering::Less => match self.left.next() {
                    Some(l) => return Some(Left(l)),
                    None => return None,
                },
                Ordering::Greater => {self.right.next();},
                Ordering::Equal => match (self.left.next(), self.right.next()) {
                    (Some(l), Some(r)) => return Some(Both(l, r)),
                    _ => return None,
                }
            }
        }
    }
}

/// An iterator adaptor that [full outer
/// joins](https://en.wikipedia.org/wiki/Join_%28SQL%29#Full_outer_join) the two base iterators in
/// ascending order. The resulting iterator contains all the records from the both input iterators.
///
/// The both base iterators must be sorted (ascending) and unique on the join key (e.g. by
/// [grouping](trait.Itertools.html#method.group_by) them, if necessary) to produce the correct results.
///
/// Iterator element type is [`EitherOrBoth<L::Item, R::Item>`](enum.EitherOrBoth.html).
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MergeJoinFullOuter<L, R, F> where
    L: Iterator,
    R: Iterator,
{
    left: Peekable<L>,
    right: Peekable<R>,
    cmp: F,
    fused: Option<Ordering>,
}

impl<L, R, F> MergeJoinFullOuter<L, R, F> where
    L: Iterator,
    R: Iterator,
{
    /// Create a `MergeJoinFullOuter` iterator.
    pub fn new<LI, RI>(left: LI, right: RI, cmp: F) -> Self
        where L: Iterator<Item=LI::Item>,
              LI: IntoIterator<IntoIter=L>,
              R: Iterator<Item=RI::Item>,
              RI: IntoIterator<IntoIter=R>,
              F: FnMut(&L::Item, &R::Item) -> Ordering
    {
        MergeJoinFullOuter {
            left: left.into_iter().peekable(),
            right: right.into_iter().peekable(),
            cmp: cmp,
            fused: None,
        }
    }
}

impl<L, R, F> Iterator for MergeJoinFullOuter<L, R, F>
    where L: Iterator,
          R: Iterator,
          F: FnMut(&L::Item, &R::Item) -> Ordering
{
    type Item = EitherOrBoth<L::Item, R::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ord = match self.fused {
                Some(o) => o,
                None => match (self.left.peek(), self.right.peek()) {
                    (Some(l), Some(r)) => (self.cmp)(l, r),
                    (Some(_), None) => {
                        self.fused = Some(Ordering::Less);
                        Ordering::Less
                    }
                    (None, Some(_)) => {
                        self.fused = Some(Ordering::Greater);
                        Ordering::Greater
                    }
                    _ => return None,
                }
            };

            match ord {
                Ordering::Less => match self.left.next() {
                    Some(l) => return Some(Left(l)),
                    None => return None,
                },
                Ordering::Greater => match self.right.next() {
                    Some(r) => return Some(Right(r)),
                    None => return None,
                },
                Ordering::Equal => match (self.left.next(), self.right.next()) {
                    (Some(l), Some(r)) => return Some(Both(l, r)),
                    _ => return None,
                }
            }
        }
    }
}
