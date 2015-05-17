//! A module of helper traits and iterators that are not intended to be used
//! directly.

use std::ops::{
    RangeFull,
    Range,
    RangeTo,
    RangeFrom
};

use std::mem;
use std::slice;

/// Apply **IntoIterator** on each element of a tuple.
pub trait IntoIteratorTuple
{
    /// Tuple of values that implement **Iterator**.
    type Output;

    /// Return a tuple of iterators.
    fn into_iterator_tuple(self) -> Self::Output;
}

/// A helper trait for (x, y, z) ++ w => (x, y, z, w),
/// used for implementing `iproduct!`.
pub trait AppendTuple<X> {
    /// Resulting tuple type
    type Result;
    /// “Append” value **x** to a tuple.
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
    #[doc(hidden)]
    pub fn new(iter: I) -> Self
    {
        FlatTuples{iter: iter}
    }
}

impl<X, T, I> Iterator for FlatTuples<I> where
    I: Iterator<Item=(T, X)>,
    T: AppendTuple<X>,
{
    type Item = T::Result;
    #[inline]
    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next().map(|(t, x)| t.append(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<X, T, I> DoubleEndedIterator for FlatTuples<I> where
    I: DoubleEndedIterator<Item=(T, X)>,
    T: AppendTuple<X>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item>
    {
        self.iter.next_back().map(|(t, x)| t.append(x))
    }
}

/// **GenericRange** is implemented by Rust's built-in range types, produced
/// by range syntax like `a..`, `..b` or `c..d`.
pub trait GenericRange {
    #[doc(hidden)]
    /// Start index (inclusive)
    fn start(&self) -> Option<usize> { None }
    #[doc(hidden)]
    /// End index (exclusive)
    fn end(&self) -> Option<usize> { None }
}


impl GenericRange for RangeFull {}

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

/// Helper trait to convert usize to floating point type.
pub trait ToFloat<F> : Copy {
    #[doc(hidden)]
    /// Convert usize to float.
    fn to_float(self) -> F;
}

impl ToFloat<f32> for usize {
    fn to_float(self) -> f32 { self as f32 }
}

impl ToFloat<f64> for usize {
    fn to_float(self) -> f64 { self as f64 }
}

/// A trait for items that can *maybe* be joined together.
pub trait MendSlice
{
    #[doc(hidden)]
    /// If the slices are contiguous, return them joined into one.
    fn mend(Self, Self) -> Result<Self, (Self, Self)>;
}

impl<'a, T> MendSlice for &'a [T]
{
    #[inline]
    fn mend(a: Self, b: Self) -> Result<Self, (Self, Self)>
    {
        unsafe {
            let a_end = a.as_ptr().offset(a.len() as isize);
            if a_end == b.as_ptr() {
                Ok(slice::from_raw_parts(a.as_ptr(), a.len() + b.len()))
            } else {
                Err((a, b))
            }
        }
    }
}

impl<'a, T> MendSlice for &'a mut [T]
{
    #[inline]
    fn mend(a: Self, b: Self) -> Result<Self, (Self, Self)>
    {
        unsafe {
            let a_end = a.as_ptr().offset(a.len() as isize);
            if a_end == b.as_ptr() {
                Ok(slice::from_raw_parts_mut(a.as_mut_ptr(), a.len() + b.len()))
            } else {
                Err((a, b))
            }
        }
    }
}

impl<'a> MendSlice for &'a str
{
    #[inline]
    fn mend(a: Self, b: Self) -> Result<Self, (Self, Self)>
    {
        unsafe {
            mem::transmute(MendSlice::mend(a.as_bytes(), b.as_bytes()))
        }
    }
}
