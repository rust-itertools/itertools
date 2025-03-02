use alloc::vec::Vec;

use crate::next_array::ArrayBuilder;

/// An iterator that groups the items in arrays of const generic size `N`.
///
/// See [`.next_array()`](crate::Itertools::next_array) for details.
#[derive(Debug, Clone)]
pub struct ArrayChunks<I: Iterator, const N: usize> {
    iter: I,
    partial: Vec<I::Item>,
}

impl<I: Iterator, const N: usize> ArrayChunks<I, N> {
    pub(crate) fn new(iter: I) -> Self {
        const {
            assert!(N > 0);
        }
        // TODO should we use iter.fuse() instead? Otherwise remainder may behave strangely
        Self {
            iter,
            partial: Vec::new(),
        }
    }

    /// Returns an iterator that yields all the items that have
    /// not been included in any of the arrays. Use this to access the
    /// leftover elements if the total number of elements yielded by
    /// the original iterator is not a multiple of `N`.
    ///
    /// If `self` is not exhausted (i.e. `next()` has not returned `None`)  
    /// then the iterator returned by `remainder()` will also include
    /// the elements that *would* have been included in the arrays
    /// produced by `next()`.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut it = (1..9).array_chunks();
    /// assert_eq!(Some([1, 2, 3]), it.next());
    /// assert_eq!(Some([4, 5, 6]), it.next());
    /// assert_eq!(None, it.next());
    /// itertools::assert_equal(it.remainder(), [7,8]);
    ///
    /// let mut it = (1..9).array_chunks();
    /// assert_eq!(Some([1, 2, 3]), it.next());
    /// itertools::assert_equal(it.remainder(), 4..9);
    /// ```
    pub fn remainder(self) -> impl Iterator<Item = I::Item> {
        self.partial.into_iter().chain(self.iter)
    }
}

impl<I: Iterator, const N: usize> Iterator for ArrayChunks<I, N> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        if !self.partial.is_empty() {
            return None;
        }
        let mut builder = ArrayBuilder::new();
        for _ in 0..N {
            if let Some(item) = self.iter.next() {
                builder.push(item);
            } else {
                break;
            }
        }
        if let Some(array) = builder.take() {
            Some(array)
        } else {
            self.partial = builder.into_vec();
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if N == 0 {
            (usize::MAX, None)
        } else {
            let (lo, hi) = self.iter.size_hint();
            (lo / N, hi.map(|hi| hi / N))
        }
    }
}
