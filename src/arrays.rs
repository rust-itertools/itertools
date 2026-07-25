use crate::Itertools;

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
}

impl<I: Iterator, const N: usize> Arrays<I, N> {
    pub(crate) fn new(iter: I) -> Self {
        const_assert_positive!(N);

        // TODO should we use iter.fuse() instead? Otherwise remainder may behave strangely
        Self { iter }
    }
}

impl<I: Iterator, const N: usize> Iterator for Arrays<I, N> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_array()
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
