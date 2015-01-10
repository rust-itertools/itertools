//! A module of helper traits and iterators that are not intended to be used
//! directly.

use std::ops::{
    FullRange,
    Range,
    RangeTo,
    RangeFrom
};


/// A helper trait for (x,y,z) ++ w => (x,y,z,w),
/// used for implementing `iproduct!` and `izip!`
pub trait AppendTuple<X> {
    type Result;
    fn append(self, x: X) -> Self::Result;
}

macro_rules! impl_append_tuple(
    () => (
        impl<T> AppendTuple<T> for () {
            type Result = (T, );
            fn append(self, x: T) -> (T, ) {
                (x, )
            }
        }
    );

    ($A:ident, $($B:ident,)*) => (
        impl_append_tuple!($($B,)*);
        #[allow(non_snake_case)]
        impl<$A, $($B,)* T> AppendTuple<T> for ($A, $($B),*) {
            type Result = ($A, $($B, )* T);
            fn append(self, x: T) -> ($A, $($B,)* T) {
                let ($A, $($B),*) = self;
                ($A, $($B,)* x)
            }
        }
    );
);

impl_append_tuple!(A, B, C, D, E, F, G, H, I, J, K, L,);

/// A helper iterator that maps an iterator of tuples like
/// `((A, B), C)` to an iterator of `(A, B, C)`.
///
/// Used by the `iproduct!()` macro.
#[derive(Clone)]
pub struct FlatTuples<I> {
    iter: I,
}

impl<I> FlatTuples<I>
{
    /// Create a new **FlatTuples**.
    pub fn new(iter: I) -> Self
    {
        FlatTuples{iter: iter}
    }
}

impl<X, T, I>
Iterator for FlatTuples<I>
    where
        I: Iterator<Item=(T, X)>,
        T: AppendTuple<X>,
{
    type Item = <T as AppendTuple<X>>::Result;
    #[inline]
    fn next(&mut self) -> Option< <Self as Iterator>::Item>
    {
        self.iter.next().map(|(t, x)| t.append(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<X, T, I> DoubleEndedIterator for FlatTuples<I>
    where
        I: DoubleEndedIterator<Item=(T, X)>,
        T: AppendTuple<X>,
{
    #[inline]
    fn next_back(&mut self) -> Option< <Self as Iterator>::Item>
    {
        self.iter.next_back().map(|(t, x)| t.append(x))
    }
}

/// **GenericRange** is implemented by Rust's built-in range types, produced
/// by range syntax like `a..`, `..b` or `c..d`.
pub trait GenericRange {
    /// Start index (inclusive)
    fn start(&self) -> Option<usize> { None }
    /// End index (exclusive)
    fn end(&self) -> Option<usize> { None }
}


impl GenericRange for FullRange {}

impl GenericRange for RangeFrom<usize> {
    fn start(&self) -> Option<usize> { Some(self.start) }
}

impl GenericRange for RangeTo<usize> {
    fn end(&self) -> Option<usize> { Some(self.end) }
}

impl GenericRange for Range<usize> {
    fn start(&self) -> Option<usize> { Some(self.start) }
    fn end(&self) -> Option<usize> { Some(self.end) }
}

