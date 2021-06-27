use std::{
    cmp::{self, Ordering},
    fmt,
    iter::FusedIterator,
};

/// Core of an iterator that merges the output of two strictly ascending iterators,
/// for instance a union or a symmetric difference.
struct MergeIterInner<I: Iterator> {
    a: I,
    b: I,
    peeked: Option<Peeked<I>>,
}

/// Benchmarks faster than wrapping both iterators in a Peekable,
/// probably because we can afford to impose a FusedIterator bound.
#[derive(Clone, Debug)]
enum Peeked<I: Iterator> {
    A(I::Item),
    B(I::Item),
}

impl<I: Iterator> MergeIterInner<I> {
    /// Creates a new core for an iterator merging a pair of sources.
    pub fn new(a: I, b: I) -> Self {
        MergeIterInner { a, b, peeked: None }
    }

    /// Returns the next pair of items stemming from the pair of sources
    /// being merged. If both returned options contain a value, that value
    /// is equal and occurs in both sources. If one of the returned options
    /// contains a value, that value doesn't occur in the other source (or
    /// the sources are not strictly ascending). If neither returned option
    /// contains a value, iteration has finished and subsequent calls will
    /// return the same empty pair.
    pub fn nexts<Cmp: Fn(&I::Item, &I::Item) -> Ordering>(
        &mut self,
        cmp: Cmp,
    ) -> (Option<I::Item>, Option<I::Item>)
    where
        I: FusedIterator,
        I::Item: fmt::Debug,
    {
        let mut a_next;
        let mut b_next;
        match self.peeked.take() {
            Some(Peeked::A(next)) => {
                a_next = Some(next);
                b_next = self.b.next();
            }
            Some(Peeked::B(next)) => {
                b_next = Some(next);
                a_next = self.a.next();
            }
            None => {
                a_next = self.a.next();
                b_next = self.b.next();
            }
        }
        if let (Some(ref a1), Some(ref b1)) = (&a_next, &b_next) {
            match cmp(a1, b1) {
                Ordering::Less => self.peeked = b_next.take().map(Peeked::B),
                Ordering::Greater => self.peeked = a_next.take().map(Peeked::A),
                Ordering::Equal => (),
            }
        }
        (a_next, b_next)
    }

    /// Returns a pair of upper bounds for the `size_hint` of the final iterator.
    pub fn lens(&self) -> (usize, usize)
    where
        I: ExactSizeIterator,
    {
        match self.peeked {
            Some(Peeked::A(_)) => (1 + self.a.len(), self.b.len()),
            Some(Peeked::B(_)) => (self.a.len(), 1 + self.b.len()),
            _ => (self.a.len(), self.b.len()),
        }
    }
}

impl<I> Clone for MergeIterInner<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    clone_fields!(a, b, peeked);
}

impl<I> fmt::Debug for MergeIterInner<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(MergeIterInner, a, b, peeked);
}

/// An iterator adaptor that merge-joins items from the two base iterators in ascending order.
///
/// See [`.union_ref()`](crate::Itertools::union_ref) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct UnionRef<I: Iterator> {
    inner: MergeIterInner<I>,
}

/// Return an iterator adaptor that merge-joins items from the two base iterators in ascending order.
///
/// See [`.union_ref()`](crate::Itertools::union_ref) for more information.
pub fn union_ref<I>(left: I, right: I) -> UnionRef<I>
where
    I: Iterator,
{
    UnionRef {
        inner: MergeIterInner::new(left, right),
    }
}

impl<I> Clone for UnionRef<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    clone_fields!(inner);
}

impl<I> fmt::Debug for UnionRef<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(UnionRef, inner);
}

impl<I> Iterator for UnionRef<I>
where
    I: Iterator + ExactSizeIterator + FusedIterator,
    I::Item: Ord + fmt::Debug,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let (a, b) = self.inner.nexts(Self::Item::cmp);
        a.or(b)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_len, b_len) = self.inner.lens();
        // We use checked add since we aren't guaranteed a set/map
        // which have a storage limit of `usize::MAX / 2`.
        (cmp::max(a_len, b_len), a_len.checked_add(b_len))
    }
}

#[derive(Clone, Debug)]
struct DoubleEndedPeeker<I: Iterator> {
    front_back: [Option<Option<I::Item>>; 2],
    iter: I,
}

impl<I> DoubleEndedPeeker<I>
where
    I: Iterator + DoubleEndedIterator,
    I::Item: Copy + fmt::Debug, // We have a &T
{
    fn new(iter: I) -> DoubleEndedPeeker<I> {
        Self {
            front_back: [None, None],
            iter,
        }
    }

    fn peek(&mut self) -> Option<I::Item> {
        self.front_back[0]
            .get_or_insert(self.iter.next())
            .or_else(|| self.front_back[1].flatten())
    }

    fn peek_last(&mut self) -> Option<I::Item> {
        self.front_back[1]
            .get_or_insert(self.iter.next_back())
            .or_else(|| self.front_back[0].flatten())
    }

    fn exact_len(&self) -> usize {
        let (low, upper) = self.size_hint();
        cmp::max(low, upper.unwrap_or_default())
    }
}

impl<I: Iterator> Iterator for DoubleEndedPeeker<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.front_back {
            [f @ Some(_), _] => f.take().flatten(),
            [_, l @ Some(_)] => {
                if let Some(n) = self.iter.next() {
                    Some(n)
                } else {
                    l.take().flatten()
                }
            }
            [None, None] => self.iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let peeked = {
            let [f, b] = &self.front_back;
            f.as_ref().map_or(0, |_| 1) + b.as_ref().map_or(0, |_| 1)
        };
        let (low, high) = self.iter.size_hint();
        (low, high.map(|h| h + peeked))
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for DoubleEndedPeeker<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.front_back {
            [_, l @ Some(_)] => l.take().flatten(),
            [f @ Some(_), _] => {
                if let Some(n) = self.iter.next_back() {
                    Some(n)
                } else {
                    f.take().flatten()
                }
            }
            [None, None] => self.iter.next_back(),
        }
    }
}

enum IntersectionInner<I: Iterator> {
    Stitch {
        // iterate similarly sized sets jointly, spotting matches along the way
        left: DoubleEndedPeeker<I>,
        right: DoubleEndedPeeker<I>,
    },
    Search {
        // iterate a small set, look up in the large set
        small: DoubleEndedPeeker<I>,
        large: DoubleEndedPeeker<I>,
    },
    Answer(Option<I::Item>), // return a specific value or emptiness
}

impl<I> Clone for IntersectionInner<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    fn clone(&self) -> Self {
        match self {
            IntersectionInner::Stitch { left, right } => IntersectionInner::Stitch {
                left: left.clone(),
                right: right.clone(),
            },
            IntersectionInner::Search { small, large } => IntersectionInner::Search {
                small: small.clone(),
                large: large.clone(),
            },
            IntersectionInner::Answer(answer) => IntersectionInner::Answer(answer.clone()),
        }
    }
}

impl<I> fmt::Debug for IntersectionInner<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntersectionInner::Stitch { left, right } => f
                .debug_struct("Stitch")
                .field("left", left)
                .field("right", right)
                .finish(),
            IntersectionInner::Search { small, large } => f
                .debug_struct("Stitch")
                .field("small", small)
                .field("large", large)
                .finish(),
            IntersectionInner::Answer(iter) => f.debug_tuple("Answer").field(iter).finish(),
        }
    }
}

/// An iterator adaptor that merge-joins items from the two base iterators in ascending order.
///
/// See [`.intersection()`](crate::Itertools::intersection) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Intersection<I: Iterator> {
    inner: IntersectionInner<I>,
}

/// Return an iterator adaptor that contains items common to both `left` and `right`.
///
/// See [`.intersection()`](crate::Itertools::intersection) for more information.
pub fn intersection<I>(left: I, right: I) -> Intersection<I>
where
    I: Iterator + DoubleEndedIterator + fmt::Debug,
    I::Item: Ord + Copy + fmt::Debug, // Items should be `&T`
{
    let mut left = DoubleEndedPeeker::new(left);
    let mut right = DoubleEndedPeeker::new(right);
    let (small_cmp, large_cmp) = {
        let (left_min, left_max) =
            if let (Some(left_min), Some(left_max)) = (left.peek(), left.peek_last()) {
                (left_min, left_max)
            } else {
                return Intersection {
                    inner: IntersectionInner::Answer(None),
                };
            };
        let (right_min, right_max) =
            if let (Some(right_min), Some(right_max)) = (right.peek(), right.peek_last()) {
                (right_min, right_max)
            } else {
                return Intersection {
                    inner: IntersectionInner::Answer(None),
                };
            };
        (left_min.cmp(&right_max), left_max.cmp(&right_min))
    };
    Intersection {
        inner: match (small_cmp, large_cmp) {
            (Ordering::Greater, _) | (_, Ordering::Less) => IntersectionInner::Answer(None),
            (Ordering::Equal, _) => IntersectionInner::Answer(left.next()),
            (_, Ordering::Equal) => IntersectionInner::Answer(left.last()),
            _ if left.exact_len() <= right.exact_len() / 16 => IntersectionInner::Search {
                small: left,
                large: right,
            },
            _ => IntersectionInner::Stitch { left, right },
        },
    }
}

impl<I> Clone for Intersection<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    clone_fields!(inner);
}

impl<I> fmt::Debug for Intersection<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(Intersection, inner);
}

impl<I> Iterator for Intersection<I>
where
    I: Iterator + FusedIterator,
    I::Item: Ord,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            IntersectionInner::Stitch { left, right } => {
                let mut a_next = left.next()?;
                let mut b_next = right.next()?;
                loop {
                    match Self::Item::cmp(&a_next, &b_next) {
                        Ordering::Less => a_next = left.next()?,
                        Ordering::Greater => b_next = right.next()?,
                        Ordering::Equal => return Some(a_next),
                    }
                }
            }
            IntersectionInner::Search { small, large } => loop {
                let next = small.next()?;
                if large.any(|b| next == b) {
                    return Some(next);
                }
            },
            IntersectionInner::Answer(answer) => answer.take(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            IntersectionInner::Stitch { left, .. } => left.size_hint(),
            IntersectionInner::Search { small, .. } => small.size_hint(),
            IntersectionInner::Answer(None) => (0, Some(0)),
            IntersectionInner::Answer(Some(_)) => (1, Some(1)),
        }
    }
}

enum DifferenceInner<I: Iterator> {
    /// Iterate all of `left` and some of `right`, spotting matches along the way.
    Stitch {
        left_iter: DoubleEndedPeeker<I>,
        right_iter: DoubleEndedPeeker<I>,
    },
    /// Iterate `left`, looking up if contained in `right`.
    Search {
        left_iter: DoubleEndedPeeker<I>,
        right_iter: DoubleEndedPeeker<I>,
    },
    Iterate(DoubleEndedPeeker<I>), // simply produce all values in `left`
}

impl<I> Clone for DifferenceInner<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    fn clone(&self) -> Self {
        match self {
            DifferenceInner::Stitch {
                left_iter,
                right_iter,
            } => DifferenceInner::Stitch {
                left_iter: left_iter.clone(),
                right_iter: right_iter.clone(),
            },
            DifferenceInner::Search {
                left_iter,
                right_iter,
            } => DifferenceInner::Search {
                left_iter: left_iter.clone(),
                right_iter: right_iter.clone(),
            },
            DifferenceInner::Iterate(iter) => DifferenceInner::Iterate(iter.clone()),
        }
    }
}

impl<I> fmt::Debug for DifferenceInner<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DifferenceInner::Stitch {
                left_iter,
                right_iter,
            } => f
                .debug_struct("Stitch")
                .field("left_iter", left_iter)
                .field("right_iter", right_iter)
                .finish(),
            DifferenceInner::Search {
                left_iter,
                right_iter,
            } => f
                .debug_struct("Stitch")
                .field("left_iter", left_iter)
                .field("right_iter", right_iter)
                .finish(),
            DifferenceInner::Iterate(iter) => f.debug_tuple("Iterate").field(iter).finish(),
        }
    }
}

/// An iterator adaptor that returns the items that represent the difference.
///
/// See [`.difference()`](crate::Itertools::difference) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Difference<I: Iterator> {
    inner: DifferenceInner<I>,
}

/// Return an iterator adaptor that returns the items that represent the difference.
///
/// See [`.difference()`](crate::Itertools::difference) for more information.
pub fn difference<I>(left: I, right: I) -> Difference<I>
where
    I: Iterator + DoubleEndedIterator,
    I::Item: Ord + Copy + fmt::Debug, // The item should be a &T
{
    let mut left = DoubleEndedPeeker::new(left);
    let mut right = DoubleEndedPeeker::new(right);
    let (small_cmp, large_cmp) = {
        let (left_min, left_max) =
            if let (Some(left_min), Some(left_max)) = (left.peek(), left.peek_last()) {
                (left_min, left_max)
            } else {
                return Difference {
                    inner: DifferenceInner::Iterate(left),
                };
            };
        let (right_min, right_max) =
            if let (Some(right_min), Some(right_max)) = (right.peek(), right.peek_last()) {
                (right_min, right_max)
            } else {
                return Difference {
                    inner: DifferenceInner::Iterate(left),
                };
            };
        (left_min.cmp(&right_max), left_max.cmp(&right_min))
    };
    Difference {
        inner: match (small_cmp, large_cmp) {
            (Ordering::Greater, _) | (_, Ordering::Less) => DifferenceInner::Iterate(left),
            (Ordering::Equal, _) => {
                left.next();
                DifferenceInner::Iterate(left)
            }
            (_, Ordering::Equal) => {
                left.next_back();
                DifferenceInner::Iterate(left)
            }
            _ if dbg!(left.exact_len()) <= dbg!(right.exact_len() / 16) => {
                DifferenceInner::Search {
                    left_iter: left,
                    right_iter: right,
                }
            }
            _ => DifferenceInner::Stitch {
                left_iter: left,
                right_iter: right,
            },
        },
    }
}

impl<I> Clone for Difference<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    clone_fields!(inner);
}

impl<I> fmt::Debug for Difference<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(Difference, inner);
}

impl<I> Iterator for Difference<I>
where
    I: Iterator + DoubleEndedIterator + FusedIterator + fmt::Debug,
    I::Item: Ord + Copy + fmt::Debug,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        println!("{:?}", self.inner);
        match &mut self.inner {
            DifferenceInner::Stitch {
                left_iter,
                right_iter,
            } => {
                let mut left_next = left_iter.next()?;
                loop {
                    match right_iter
                        .peek()
                        .map_or(Ordering::Less, |right_next| left_next.cmp(&right_next))
                    {
                        Ordering::Less => return Some(left_next),
                        Ordering::Equal => {
                            left_next = left_iter.next()?;
                            right_iter.next();
                        }
                        Ordering::Greater => {
                            right_iter.next();
                        }
                    }
                }
            }
            DifferenceInner::Search {
                left_iter,
                right_iter,
            } => loop {
                let left_next = left_iter.next()?;
                if !right_iter.any(|a| a == left_next) {
                    return Some(left_next);
                }
            },
            DifferenceInner::Iterate(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (self_len, other_len) = match &self.inner {
            DifferenceInner::Stitch {
                left_iter,
                right_iter,
            } => (left_iter.exact_len(), right_iter.exact_len()),
            DifferenceInner::Search {
                left_iter,
                right_iter: other_set,
            } => (left_iter.exact_len(), other_set.exact_len()),
            DifferenceInner::Iterate(iter) => (iter.exact_len(), 0),
        };
        (self_len.saturating_sub(other_len), Some(self_len))
    }
}

/// An iterator adaptor that returns the values representing the symmetric difference.
///
/// See [`.symmetric_difference()`](crate::Itertools::symetric_difference) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct SymmetricDifference<I: Iterator> {
    inner: MergeIterInner<I>,
}

/// Return an iterator adaptor that returns the values representing the symmetric difference.
///
/// See [`.symmetric_difference()`](crate::Itertools::symmetric_difference) for more information.
pub fn symmetric_difference<I>(left: I, right: I) -> SymmetricDifference<I>
where
    I: Iterator,
    I::Item: Ord,
{
    SymmetricDifference {
        inner: MergeIterInner::new(left, right),
    }
}

impl<I> Iterator for SymmetricDifference<I>
where
    I: Iterator + ExactSizeIterator + FusedIterator,
    I::Item: Ord + fmt::Debug,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a, b) = self.inner.nexts(Self::Item::cmp);
            if a.as_ref().and(b.as_ref()).is_none() {
                return a.or(b);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a, b) = self.inner.lens();
        // We use checked add since we aren't guaranteed a set/map
        // which have a storage limit of `usize::MAX / 2`.
        (0, a.checked_add(b))
    }
}
