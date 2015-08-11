use std::cmp;

use misc::Slice;

// Note: There are different ways to implement ZipSlices.
// This version performed the best in benchmarks.
//
// I also implemented a version with three pointes (tptr, tend, uptr),
// that mimiced slice::Iter and only checked bounds by using tptr == tend,
// but that was inferior to this solution.

/// An iterator which iterates two slices simultaneously.
///
/// `ZipSlices` acts like a double-ended `.zip()` iterator, but more efficiently.
///
/// Note that elements past the end of the shortest of the two slices are ignored.
///
/// Iterator element type for `ZipSlices<T, U>` is `(T::Item, U::Item)`. For example,
/// for a `ZipSlices<&'a [A], &'b mut [B]>`, the element type is `(&'a A, &'b mut B)`.
#[derive(Clone)]
pub struct ZipSlices<T, U> {
    t: T,
    u: U,
    len: usize,
    index: usize,
}

impl<'a, 'b, A, B> ZipSlices<&'a [A], &'b [B]> {
    /// Create a new `ZipSlices` from slices `a` and `b`.
    ///
    /// Act like a double-ended `.zip()` iterator, but more efficiently.
    ///
    /// Note that elements past the end of the shortest of the two slices are ignored.
    #[inline(always)]
    pub fn new(a: &'a [A], b: &'b [B]) -> Self {
        let minl = cmp::min(a.len(), b.len());
        ZipSlices {
            t: a,
            u: b,
            len: minl,
            index: 0,
        }
    }
}

impl<T, U> ZipSlices<T, U>
    where T: Slice, U: Slice
{
    /// Create a new `ZipSlices` from slices `a` and `b`.
    ///
    /// Act like a double-ended `.zip()` iterator, but more efficiently.
    ///
    /// Note that elements past the end of the shortest of the two slices are ignored.
    #[inline(always)]
    pub fn from_slices(a: T, b: U) -> Self {
        let minl = cmp::min(a.len(), b.len());
        ZipSlices {
            t: a,
            u: b,
            len: minl,
            index: 0,
        }
    }
}

impl<T, U> Iterator for ZipSlices<T, U>
    where T: Slice, U: Slice
{
    type Item = (T::Item, U::Item);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.index >= self.len {
                None
            } else {
                let i = self.index;
                self.index += 1;
                Some((
                    self.t.get_unchecked(i),
                    self.u.get_unchecked(i)))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len - self.index;
        (len, Some(len))
    }
}

impl<T, U> DoubleEndedIterator for ZipSlices<T, U>
    where T: Slice, U: Slice
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.index >= self.len {
                None
            } else {
                self.len -= 1;
                let i = self.len;
                Some((
                    self.t.get_unchecked(i),
                    self.u.get_unchecked(i)))
            }
        }
    }
}

impl<T, U> ExactSizeIterator for ZipSlices<T, U> where T: Slice, U: Slice { }

