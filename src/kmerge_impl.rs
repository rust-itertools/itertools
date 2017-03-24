
use size_hint;
use Itertools;

use std::cmp::Ordering;
use std::mem::replace;
use std::collections::BinaryHeap;
#[cfg(feature = "nightly")]
use std::collections::binary_heap::PeekMut;

macro_rules! clone_fields {
    ($name:ident, $base:expr, $($field:ident),+) => (
        $name {
            $(
                $field : $base . $field .clone()
            ),*
        }
    );
}

/// Head element and Tail iterator pair
///
/// `PartialEq`, `Eq`, `PartialOrd` and `Ord` are implemented by comparing sequences based on
/// first items (which are guaranteed to exist).
///
/// The meanings of `PartialOrd` and `Ord` are reversed so as to turn the heap used in
/// `KMerge` into a min-heap.
struct HeadTail<I>
    where I: Iterator
{
    head: I::Item,
    tail: I,
}

impl<I> HeadTail<I>
    where I: Iterator
{
    /// Constructs a `HeadTail` from an `Iterator`. Returns `None` if the `Iterator` is empty.
    fn new(mut it: I) -> Option<HeadTail<I>> {
        let head = it.next();
        head.map(|h| {
            HeadTail {
                head: h,
                tail: it,
            }
        })
    }

    /// Get the next element and update `head`, returning the old head in `Some`.
    ///
    /// Returns `None` when the tail is exhausted (only `head` then remains).
    fn next(&mut self) -> Option<I::Item> {
        if let Some(next) = self.tail.next() {
            Some(replace(&mut self.head, next))
        } else {
            None
        }
    }

    /// Hints at the size of the sequence, same as the `Iterator` method.
    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::add_scalar(self.tail.size_hint(), 1)
    }
}

impl<I> Clone for HeadTail<I>
    where I: Iterator + Clone,
          I::Item: Clone
{
    fn clone(&self) -> Self {
        clone_fields!(HeadTail, self, head, tail)
    }
}

impl<I> PartialEq for HeadTail<I>
    where I: Iterator,
          I::Item: PartialEq
{
    fn eq(&self, other: &HeadTail<I>) -> bool {
        self.head.eq(&other.head)
    }
}

impl<I> Eq for HeadTail<I>
    where I: Iterator,
          I::Item: Eq
{}

impl<I> PartialOrd for HeadTail<I>
    where I: Iterator,
          I::Item: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.head.partial_cmp(&self.head)
    }

    fn lt(&self, other: &Self) -> bool {
        other.head.lt(&self.head)
    }

    fn le(&self, other: &Self) -> bool {
        other.head.le(&self.head)
    }

    fn gt(&self, other: &Self) -> bool {
        other.head.gt(&self.head)
    }

    fn ge(&self, other: &Self) -> bool {
        other.head.ge(&self.head)
    }
}

impl<I> Ord for HeadTail<I>
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
/// See [`.kmerge()`](../trait.Itertools.html#method.kmerge) for more information.
pub struct KMerge<I>
    where I: Iterator
{
    heap: BinaryHeap<HeadTail<I>>,
}

/// Create an iterator that merges elements of the contained iterators.
///
/// Equivalent to `i.into_iter().kmerge()`.
///
/// ```
/// use itertools::kmerge;
///
/// for elt in kmerge(vec![vec![0, 2, 4], vec![1, 3, 5], vec![6, 7]]) {
///     /* loop body */
/// }
/// ```
pub fn kmerge<I>(iterable: I) -> KMerge<<I::Item as IntoIterator>::IntoIter>
    where I: IntoIterator,
          I::Item: IntoIterator,
          <<I as IntoIterator>::Item as IntoIterator>::Item: Ord
{
    let iter = iterable.into_iter();
    let (lower, _) = iter.size_hint();
    let mut heap = BinaryHeap::with_capacity(lower);
    heap.extend(iter.filter_map(|it| HeadTail::new(it.into_iter())));
    KMerge { heap: heap }
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

    #[cfg(feature = "nightly")]
    fn next(&mut self) -> Option<Self::Item> {
        self.heap.peek_mut().and_then(|mut iter| {
            iter.next().or_else(|| {
                Some(PeekMut::pop(iter).head)
            })
        })
    }

    #[cfg(not(feature = "nightly"))]
    fn next(&mut self) -> Option<Self::Item> {
        let mut pop_it = false;
        let mut val = self.heap.peek_mut().and_then(|mut iter| {
            iter.next().or_else(|| {
                pop_it = true;
                None // place holder
            })
        });
        if pop_it {
            val = Some(self.heap.pop().unwrap().head);
        }
        val
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.heap.iter()
                 .map(|i| i.size_hint())
                 .fold1(size_hint::add)
                 .unwrap_or((0, Some(0)))
    }
}

