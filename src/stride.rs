//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

use std::kinds;
use std::mem;
use std::num;
use std::ptr;
use std::fmt;

/// Similar to the slice iterator, but with a certain number of steps
/// (stride) skipped per iteration.
///
/// Does not support zero-sized `A`.
///
/// Iterator element type is `&'a A`
pub struct Stride<'a, A> {
    begin: *const A,
    // Unlike the slice iterator, end is inclusive and the last
    // pointer we will visit. This makes it possible to have
    // safe stride iterators for columns in matrices etc.
    end: *const A,
    stride: int,
    life: kinds::marker::ContravariantLifetime<'a>,
}

impl<'a, A> Stride<'a, A>
{
    /// Create Stride iterator from a slice and the element step count
    ///
    /// ## Example
    ///
    /// ```
    /// let xs = [0i, 1, 2, 3, 4, 5];
    /// let mut iter = Stride::from_slice(xs.as_slice(), 2);
    /// ```
    pub fn from_slice(xs: &'a [A], step: uint) -> Stride<'a, A>
    {
        assert!(step != 0);
        assert!(mem::size_of::<A>() != 0);
        let mut begin = ptr::null();
        let mut end = ptr::null();
        let (d, r) = num::div_rem(xs.len(), step);
        let nelem = d + if r > 0 { 1 } else { 0 };
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
    pub unsafe fn from_ptrs(begin: *const A, end: *const A, stride: int) -> Stride<'a, A>
    {
        Stride {
            begin: begin,
            end: end,
            stride: stride,
            life: kinds::marker::ContravariantLifetime,
        }
    }

    /// Swap the being and end pointer and reverse the stride,
    /// in effect reversing the iterator.
    #[inline]
    pub fn swap_ends(&mut self) {
        if !self.begin.is_null() {
            mem::swap(&mut self.begin, &mut self.end);
            self.stride = -self.stride;
        }
    }
}

impl<'a, A> Iterator<&'a A> for Stride<'a, A>
{
    #[inline]
    fn next(&mut self) -> Option<&'a A>
    {
        if self.begin.is_null() {
            None
        } else {
            unsafe {
                let elt: &'a A = mem::transmute(self.begin);
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
                / mem::size_of::<A>() as int + 1;
        }

        (len as uint, Some(len as uint))
    }
}

impl<'a, A> DoubleEndedIterator<&'a A> for Stride<'a, A>
{
    #[inline]
    fn next_back(&mut self) -> Option<&'a A>
    {
        if self.begin.is_null() {
            None
        } else {
            unsafe {
                let elt: &'a A = mem::transmute(self.end);
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

impl<'a, A> ExactSize<&'a A> for Stride<'a, A> { }

impl<'a, A> Index<uint, A> for Stride<'a, A>
{
    fn index<'b>(&'b self, i: &uint) -> &'b A
    {
        assert!(*i < self.size_hint().val0());
        unsafe {
            let ptr = self.begin.offset(self.stride * (*i as int));
            mem::transmute(ptr)
        }
    }
}

impl<'a, A: fmt::Show> fmt::Show for Stride<'a, A>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let it = *self;
        try!(write!(f, "["));
        for (i, elt) in it.enumerate() {
            if i != 0 {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{}", *elt));
        }
        write!(f, "]")
    }
}

impl<'a, A> Clone for Stride<'a, A>
{
    fn clone(&self) -> Stride<'a, A>
    {
        *self
    }
}
