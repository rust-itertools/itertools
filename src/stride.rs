//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

use std::kinds;
use std::mem;
use std::ptr;
use std::fmt;

use std::num::Saturating;

/// Similar to the slice iterator, but with a certain number of steps
/// (stride) skipped per iteration.
///
/// Does not support zero-sized `T`.
pub struct Stride<'a, T> {
    begin: *const T,
    // Unlike the slice iterator, end is inclusive and the last
    // pointer we will visit. This makes it possible to have
    // safe stride iterators for columns in matrices etc.
    end: *const T,
    stride: int,
    life: kinds::marker::ContravariantLifetime<'a>,
}

impl<'a, T> Stride<'a, T>
{
    /// Create Stride iterator from a slice and the element step count
    ///
    /// ## Example
    ///
    /// ```
    /// let xs = [0i, 1, 2, 3, 4, 5];
    /// let mut iter = Stride::from_slice(xs.as_slice(), 2);
    /// ```
    pub fn from_slice(xs: &'a [T], step: uint) -> Stride<'a, T>
    {
        assert!(step != 0);
        let mut begin = ptr::null();
        let mut end = ptr::null();
        let nelem = xs.len().saturating_add(step - 1) / step;
        unsafe {
            if nelem != 0 {
                begin = xs.as_ptr();
                end = begin.offset(((nelem - 1) * step) as int);
            }
            Stride::from_ptrs(begin, end, step as int)
        }
    }

    /// Create Stride iterator from raw pointers from the *inclusive*
    /// pointer range [begin, end].
    ///
    /// **Note:** `end` **must** be a whole number of `stride` steps away
    /// from `begin`
    pub unsafe fn from_ptrs(begin: *const T, end: *const T, stride: int) -> Stride<'a, T>
    {
        Stride {
            begin: begin,
            end: end,
            stride: stride,
            life: kinds::marker::ContravariantLifetime,
        }
    }

}

impl<'a, T> Iterator<&'a T> for Stride<'a, T>
{
    #[inline]
    fn next(&mut self) -> Option<&'a T>
    {
        if self.begin.is_null() {
            None
        } else {
            unsafe {
                let elt: &'a T = mem::transmute(self.begin);
                if self.begin == self.end {
                    self.begin = ptr::null();
                } else {
                    self.begin = self.begin.offset(self.stride);
                }
                Some(elt)
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>)
    {
        let len;
        if self.begin.is_null() {
            len = 0;
        } else {
            len = (self.end as uint - self.begin as uint) as int / self.stride
                / mem::size_of::<T>() as int + 1;
        }

        (len as uint, Some(len as uint))
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for Stride<'a, T>
{
    #[inline]
    fn next_back(&mut self) -> Option<&'a T>
    {
        if self.begin.is_null() {
            None
        } else {
            unsafe {
                let elt: &'a T = mem::transmute(self.end);
                if self.begin == self.end {
                    self.begin = ptr::null();
                } else {
                    self.end = self.end.offset(-self.stride);
                }
                Some(elt)
            }
        }
    }
}

impl<'a, T> ExactSize<&'a T> for Stride<'a, T> { }

impl<'a, T> Index<uint, T> for Stride<'a, T>
{
    fn index<'b>(&'b self, i: &uint) -> &'b T
    {
        assert!(*i < self.size_hint().val0());
        unsafe {
            let ptr = self.begin.offset(self.stride * (*i as int));
            mem::transmute(ptr)
        }
    }
}

impl<'a, T: fmt::Show> fmt::Show for Stride<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let mut it = *self;
        let mut res = write!(f, "[");
        for elt in it {
            res = res.and(write!(f, "{}, ", *elt));
        }
        res.and(write!(f, "]"))
    }
}

impl<'a, T> Clone for Stride<'a, T>
{
    fn clone(&self) -> Stride<'a, T>
    {
        *self
    }
}
