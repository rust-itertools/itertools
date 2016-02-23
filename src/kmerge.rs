
use size_hint;

use std::cmp::Ordering;
use std::collections::BinaryHeap;

macro_rules! clone_fields {
    ($name:ident, $base:expr, $($field:ident),+) => (
        $name {
            $(
                $field : $base . $field .clone()
            ),*
        }
    );
}

/// A non-empty sequence
///
/// `PartialEq`, `Eq`, `PartialOrd` and `Ord` are implemented by comparing sequences based on
/// first items (which are guaranteed to exist).
///
/// The meanings of `PartialOrd` and `Ord` are reversed so as to turn the `BinaryHeap` used in
/// `KMerge` into a min-heap.
struct NonEmpty<I>
    where I: Iterator
{
    head: I::Item,
    tail: I,
}

impl<I> NonEmpty<I>
    where I: Iterator
{
    /// Constructs a `NonEmpty` from an `Iterator`. Returns `None` if the `Iterator` is empty.
    fn new(mut it: I) -> Option<NonEmpty<I>> {
        let head = it.next();
        head.map(|h| {
            NonEmpty {
                head: h,
                tail: it,
            }
        })
    }

    /// Returns the next item in the sequence. If more items remain, the remainder of the sequence
    /// is returned as well, otherwise `None`.
    fn next(self) -> (I::Item, Option<Self>) {
        (self.head, NonEmpty::new(self.tail))
    }

    /// Hints at the size of the sequence, same as the `Iterator` method.
    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::add_scalar(self.tail.size_hint(), 1)
    }
}

impl<I> Clone for NonEmpty<I>
    where I: Iterator + Clone,
          I::Item: Clone
{
    fn clone(&self) -> Self {
        clone_fields!(NonEmpty, self, head, tail)
    }
}

impl<I> PartialEq for NonEmpty<I>
    where I: Iterator,
          I::Item: PartialEq
{
    fn eq(&self, other: &NonEmpty<I>) -> bool {
        self.head.eq(&other.head)
    }
}

impl<I> Eq for NonEmpty<I>
    where I: Iterator,
          I::Item: Eq
{}

impl<I> PartialOrd for NonEmpty<I>
    where I: Iterator,
          I::Item: PartialOrd
{
    fn partial_cmp(&self, other: &NonEmpty<I>) -> Option<Ordering> {
        other.head.partial_cmp(&self.head)
    }

    fn lt(&self, other: &NonEmpty<I>) -> bool {
        other.head.lt(&self.head)
    }

    fn le(&self, other: &NonEmpty<I>) -> bool {
        other.head.le(&self.head)
    }

    fn gt(&self, other: &NonEmpty<I>) -> bool {
        other.head.gt(&self.head)
    }

    fn ge(&self, other: &NonEmpty<I>) -> bool {
        other.head.ge(&self.head)
    }
}

impl<I> Ord for NonEmpty<I>
    where I: Iterator,
          I::Item: Ord
{
    fn cmp(&self, other: &Self) -> Ordering {
        other.head.cmp(&self.head)
    }
}

/// An iterator adaptor that merges an abitrary number of base iterators in ascending order.
/// If all base iterators are sorted (ascending), the result is sorted.
///
/// Iterator element type is `I::Item`.
///
/// See [*.kmerge()*](trait.Itertools.html#method.kmerge) for more information.
pub struct KMerge<I>
    where I: Iterator
{
    heap: BinaryHeap<NonEmpty<I>>,
}

/// Create a `KMerge` iterator.
pub fn kmerge_new<I>(it: I) -> KMerge<<I::Item as IntoIterator>::IntoIter>
    where I: Iterator,
          I::Item: IntoIterator,
          <<I as Iterator>::Item as IntoIterator>::Item: Ord
{
    KMerge { heap: it.filter_map(|it| NonEmpty::new(it.into_iter())).collect() }
}

impl<I> Clone for KMerge<I>
    where I: Iterator + Clone,
          I::Item: Clone
{
    fn clone(&self) -> KMerge<I> {
        clone_fields!(KMerge, self, heap)
    }
}

impl<I> Iterator for KMerge<I>
    where I: Iterator,
          I::Item: Ord
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.pop().map(|p| {
            let (h, t) = p.next();
            t.map(|t| self.heap.push(t));
            h
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if !self.heap.is_empty() {
            let mut it = self.heap.iter();
            let sh0 = it.next().unwrap().size_hint();
            it.fold(sh0, |acc, it| size_hint::add(acc, it.size_hint()))
        } else {
            (0, Some(0))
        }
    }
}

