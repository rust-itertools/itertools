use std::collections::BinaryHeap;
use std::cmp::Ord;

pub(crate) fn k_smallest<T: Ord, I: Iterator<Item = T>>(mut iter: I, k: usize) -> BinaryHeap<T> {
    if k == 0 { return BinaryHeap::new(); }

    let mut heap = iter.by_ref().take(k).collect::<BinaryHeap<_>>();

    for i in iter {
        debug_assert_eq!(heap.len(), k);
        // Guaranteed not-None, since we keep exactly k>0 elements in the heap.
        let mut lorgest = heap.peek_mut().unwrap();
        // Equivalent to heap.push(min(i, heap.pop())) but more efficient.
        if *lorgest > i { *lorgest = i; }
    }

    heap
}
