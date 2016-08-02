
use size_hint;
use Itertools;

use std::cmp::Ordering;
use std::mem::replace;

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

/// Make `data` a heap (max-heap w.r.t T's Ord).
fn heapify<T: Ord>(data: &mut [T]) {
    for i in (0..data.len() / 2).rev() {
        sift_down(data, i);
    }
}

/// Sift down element at `index` (`heap` is a max-heap wrt T's Ord).
fn sift_down<T: Ord>(heap: &mut [T], index: usize) {
    debug_assert!(index <= heap.len());
    let mut pos = index;
    let mut child = 2 * pos + 1;
    // the `pos` conditional is to avoid a bounds check
    while pos < heap.len() && child < heap.len() {
        let right = child + 1;

        // pick the bigger of the two children
        if right < heap.len() && heap[child] < heap[right] {
            child = right;
        }

        // sift down is done if we are already in order
        if heap[pos] >= heap[child] {
            return;
        }
        heap.swap(pos, child);
        pos = child;
        child = 2 * pos + 1;
    }
}

/// An iterator adaptor that merges an abitrary number of base iterators in ascending order.
/// If all base iterators are sorted (ascending), the result is sorted.
///
/// Iterator element type is `I::Item`.
///
/// See [`.kmerge()`](trait.Itertools.html#method.kmerge) for more information.
pub struct KMerge<I>
    where I: Iterator
{
    heap: Vec<HeadTail<I>>,
}

/// Create a `KMerge` iterator.
pub fn kmerge_new<I>(iter: I) -> KMerge<<I::Item as IntoIterator>::IntoIter>
    where I: Iterator,
          I::Item: IntoIterator,
          <<I as Iterator>::Item as IntoIterator>::Item: Ord
{
    let (lower, _) = iter.size_hint();
    let mut heap = Vec::with_capacity(lower);
    heap.extend(iter.filter_map(|it| HeadTail::new(it.into_iter())));
    heapify(&mut heap);
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

    fn next(&mut self) -> Option<Self::Item> {
        if self.heap.is_empty() {
            return None;
        }
        let result = if let Some(next) = self.heap[0].next() {
            next
        } else {
            self.heap.swap_remove(0).head
        };
        sift_down(&mut self.heap, 0);
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.heap.iter()
                 .map(|i| i.size_hint())
                 .fold1(size_hint::add)
                 .unwrap_or((0, Some(0)))
    }
}

