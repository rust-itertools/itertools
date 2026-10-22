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

        // TODO should we use iter.fuse() instead?
        Self { iter }
    }
}

impl<I: Iterator, const N: usize> Iterator for Arrays<I, N> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_array()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // also verified in `new()`
        const_assert_positive!(N);
        let (lo, hi) = self.iter.size_hint();
        (lo / N, hi.map(|hi| hi / N))
    }
}

impl<I: ExactSizeIterator, const N: usize> ExactSizeIterator for Arrays<I, N> {}
