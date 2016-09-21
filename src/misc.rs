
macro_rules! impl_cons_iter(
    () => ();

    ($A:ident, $($B:ident,)*) => (
        impl_cons_iter!($($B,)*);
        #[allow(non_snake_case)]
        impl<X, Iter, $($B),*> Iterator for FlatTuples<Iter, (($($B,)*), X)>
            where Iter: Iterator<Item = (($($B,)*), X)>,
        {
            type Item = ($($B,)* X, );
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|(($($B,)*), x)| ($($B,)* x, ))
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        #[allow(non_snake_case)]
        impl<X, Iter, $($B),*> DoubleEndedIterator for FlatTuples<Iter, (($($B,)*), X)>
            where Iter: DoubleEndedIterator<Item = (($($B,)*), X)>,
        {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|(($($B,)*), x)| ($($B,)* x, ))
            }
        }

    );
);

impl_cons_iter!(A, B, C, D, E, F, G, H, I, J, K, L,);

/// A helper iterator that maps an iterator of tuples like
/// `((A, B), C)` to an iterator of `(A, B, C)`.
///
/// Used by the `iproduct!()` macro.
pub struct FlatTuples<I, J>
    where I: Iterator<Item=J>,
{
    iter: I,
}

impl<I, J> Clone for FlatTuples<I, J>
    where I: Clone + Iterator<Item=J>,
{
    fn clone(&self) -> Self {
        FlatTuples {
            iter: self.iter.clone(),
        }
    }
}

impl<I, J> FlatTuples<I, J>
    where I: Iterator<Item=J>
{
    /// Create a new `FlatTuples`.
    #[doc(hidden)]
    pub fn new(iter: I) -> Self {
        FlatTuples { iter: iter }
    }
}
