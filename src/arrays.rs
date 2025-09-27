use alloc::vec::Vec;

use crate::next_array::ArrayBuilder;

macro_rules! const_assert_positive {
    ($N: ty) => {
        trait StaticAssert<const N: usize> {
            const ASSERT: bool;
        }

        impl<const N: usize> StaticAssert<N> for () {
            const ASSERT: bool = {
                assert!(N > 0);
                true
            };
        }

        assert!(<() as StaticAssert<N>>::ASSERT);
    };
}

/// An iterator that groups the items in arrays of const generic size `N`.
///
/// See [`.next_array()`](crate::Itertools::next_array) for details.
#[derive(Debug, Clone)]
pub struct Arrays<I: Iterator, const N: usize> {
    iter: I,
    partial: Vec<I::Item>,
}

impl<I: Iterator, const N: usize> Arrays<I, N> {
    pub(crate) fn new(iter: I) -> Self {
        const_assert_positive!(N);

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
    /// let mut it = (1..9).arrays();
    /// assert_eq!(Some([1, 2, 3]), it.next());
    /// assert_eq!(Some([4, 5, 6]), it.next());
    /// assert_eq!(None, it.next());
    /// itertools::assert_equal(it.remainder(), [7,8]);
    ///
    /// let mut it = (1..9).arrays();
    /// assert_eq!(Some([1, 2, 3]), it.next());
    /// itertools::assert_equal(it.remainder(), 4..9);
    /// ```
    pub fn remainder(self) -> impl Iterator<Item = I::Item> {
        self.partial.into_iter().chain(self.iter)
    }
}

impl<I: Iterator, const N: usize> Iterator for Arrays<I, N> {
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

impl<I: ExactSizeIterator, const N: usize> ExactSizeIterator for Arrays<I, N> {}

#[cfg(test)]
mod tests {
    use crate::Itertools;

    fn exact_size_helper(it: impl Iterator) {
        let (lo, hi) = it.size_hint();
        let count = it.count();
        assert_eq!(lo, count);
        assert_eq!(hi, Some(count));
    }

    #[test]
    fn exact_size_not_divisible() {
        let it = (0..10).array_chunks::<3>();
        exact_size_helper(it);
    }

    #[test]
    fn exact_size_after_next() {
        let mut it = (0..10).array_chunks::<3>();
        _ = it.next();
        exact_size_helper(it);
    }

    #[test]
    fn exact_size_divisible() {
        let it = (0..10).array_chunks::<5>();
        exact_size_helper(it);
    }
}
