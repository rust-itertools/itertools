use std::cmp;

// Note: There are different ways to implement ZipSlices.
// This version performed the best in benchmarks.
//
// I also implemented a version with three pointes (tptr, tend, uptr),
// that mimiced slice::Iter and only checked bounds by using tptr == tend,
// but that was inferior to this solution.

/// `ZipSlices`
///
/// Iterator element type is `(&'a T, &'a U)`.
pub struct ZipSlices<'a, T: 'a, U :'a>
{
    t: &'a [T],
    u: &'a [U],
    len: usize,
    index: usize,
}

impl<'a, T, U> Clone for ZipSlices<'a, T, U> {
    fn clone(&self) -> Self {
        ZipSlices {
            t: self.t,
            u: self.u,
            len: self.len,
            index: self.index,
        }
    }
}

impl<'a, T, U> ZipSlices<'a, T, U> {
    /// Create a new `ZipSlices` from slices `a` and `b`.
    ///
    /// Act like a double-ended `.zip()` iterator, but more efficiently.
    ///
    /// Note that elements past the shortest of `a` or `b` are ignored.
    #[inline(always)]
    pub fn new(a: &'a [T], b: &'a [U]) -> Self {
        let minl = cmp::min(a.len(), b.len());
        ZipSlices {
            t: a,
            u: b,
            len: minl,
            index: 0,
        }
    }
}

impl<'a, T, U> Iterator for ZipSlices<'a, T, U> {
    type Item = (&'a T, &'a U);

    #[inline(always)]
    fn next(&mut self) -> Option<(&'a T, &'a U)> {
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

impl<'a, T, U> DoubleEndedIterator for ZipSlices<'a, T, U> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<(&'a T, &'a U)> {
        unsafe {
            if self.index == self.len {
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

impl<'a, T, U> ExactSizeIterator for ZipSlices<'a, T, U> { }

