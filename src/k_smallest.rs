use crate::heap::{heapify, sift_down};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Consumes a given iterator, returning the minimum elements in **ascending** order.
pub(crate) fn k_smallest_general<I, F>(iter: I, k: usize, mut comparator: F) -> Vec<I::Item>
where
    I: Iterator,
    F: FnMut(&I::Item, &I::Item) -> Ordering,
{
    if k == 0 {
        iter.last();
        return Vec::new();
    }
    if k == 1 {
        return iter.min_by(comparator).into_iter().collect();
    }
    let mut iter = iter.fuse();
    let mut storage: Vec<I::Item> = iter.by_ref().take(k).collect();

    // The shared heap puts the *least* element (w.r.t. its predicate) on top.
    // We want the **largest** item on top, so we give it the reversed comparison.
    let mut is_greater_than = move |a: &_, b: &_| comparator(a, b) == Ordering::Greater;

    // Rearrange the storage into a valid heap by reordering from the second-bottom-most layer up to the root.
    // Slightly faster than ordering on each insert, but only by a factor of lg(k).
    // The resulting heap has the **largest** item on top.
    heapify(&mut storage, &mut is_greater_than);

    iter.for_each(|val| {
        debug_assert_eq!(storage.len(), k);
        // `val < storage[0]`
        if is_greater_than(&storage[0], &val) {
            // Treating this as an push-and-pop saves having to write a sift-up implementation.
            // https://en.wikipedia.org/wiki/Binary_heap#Insert_then_extract
            storage[0] = val;
            // We retain the smallest items we've seen so far, but ordered largest first so we can drop the largest efficiently.
            sift_down(&mut storage, 0, &mut is_greater_than);
        }
    });

    // Ultimately the items need to be in least-first, strict order, but the heap is currently largest-first.
    // To achieve this, repeatedly,
    // 1) "pop" the largest item off the heap into the tail slot of the underlying storage,
    // 2) shrink the logical size of the heap by 1,
    // 3) restore the heap property over the remaining items.
    let mut heap = &mut storage[..];
    while heap.len() > 1 {
        let last_idx = heap.len() - 1;
        heap.swap(0, last_idx);
        // Sifting over a truncated slice means that the sifting will not disturb already popped elements.
        heap = &mut heap[..last_idx];
        sift_down(heap, 0, &mut is_greater_than);
    }

    storage
}

pub(crate) fn k_smallest_relaxed_general<I, F>(iter: I, k: usize, mut comparator: F) -> Vec<I::Item>
where
    I: Iterator,
    F: FnMut(&I::Item, &I::Item) -> Ordering,
{
    if k == 0 {
        iter.last();
        return Vec::new();
    }

    let mut iter = iter.fuse();
    let mut buf = iter.by_ref().take(2 * k).collect::<Vec<_>>();

    if buf.len() < k {
        buf.sort_unstable_by(&mut comparator);
        return buf;
    }

    buf.select_nth_unstable_by(k - 1, &mut comparator);
    buf.truncate(k);

    iter.for_each(|val| {
        if comparator(&val, &buf[k - 1]) != Ordering::Less {
            return;
        }

        assert_ne!(buf.len(), buf.capacity());
        buf.push(val);

        if buf.len() == 2 * k {
            buf.select_nth_unstable_by(k - 1, &mut comparator);
            buf.truncate(k);
        }
    });

    buf.sort_unstable_by(&mut comparator);
    buf.truncate(k);
    buf
}

#[inline]
pub(crate) fn key_to_cmp<T, K, F>(mut key: F) -> impl FnMut(&T, &T) -> Ordering
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    move |a, b| key(a).cmp(&key(b))
}
