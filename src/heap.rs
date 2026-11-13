//! Binary-heap helpers on slices, parameterized by a "less than" predicate.
//!
//! The heap is a min-heap w.r.t. `less_than`: the root is the "least" element.

/// Make `data` a heap (min-heap w.r.t the sorting).
pub(crate) fn heapify<T, S>(data: &mut [T], mut less_than: S)
where
    S: FnMut(&T, &T) -> bool,
{
    for i in (0..data.len() / 2).rev() {
        sift_down(data, i, &mut less_than);
    }
}

/// Sift down element at `index` (`heap` is a min-heap wrt the ordering)
pub(crate) fn sift_down<T, S>(heap: &mut [T], index: usize, mut less_than: S)
where
    S: FnMut(&T, &T) -> bool,
{
    debug_assert!(index <= heap.len());
    let mut pos = index;
    let mut child = 2 * pos + 1;
    // Require the right child to be present
    // This allows to find the index of the smallest child without a branch
    // that wouldn't be predicted if present
    while child + 1 < heap.len() {
        // pick the smaller of the two children
        // use arithmetic to avoid an unpredictable branch
        child += usize::from(less_than(&heap[child + 1], &heap[child]));

        // sift down is done if we are already in order
        if !less_than(&heap[child], &heap[pos]) {
            return;
        }
        heap.swap(pos, child);
        pos = child;
        child = 2 * pos + 1;
    }
    // Check if the last (left) child was an only child
    // if it is then it has to be compared with the parent
    if child + 1 == heap.len() && less_than(&heap[child], &heap[pos]) {
        heap.swap(pos, child);
    }
}
