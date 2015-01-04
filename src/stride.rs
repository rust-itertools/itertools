//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

use std::fmt;
use std::kinds;
use std::mem;
use std::num;
use std::ops::{Index, IndexMut};

/// Stride is similar to the slice iterator, but with a certain number of steps
/// (the stride) skipped per iteration.
///
/// Stride does not support zero-sized types for `A`.
///
/// Iterator element type is `&'a A`.
pub struct Stride<'a, A> {
    /// base pointer -- does not change during iteration
    begin: *const A,
    /// current offset from begin
    offset: int,
    /// offset where we end (exclusive end).
    end: int,
    stride: int,
    life: kinds::marker::ContravariantLifetime<'a>,
}

impl<'a, A> Copy for Stride<'a, A> {}

/// StrideMut is like Stride, but with mutable elements.
///
/// Iterator element type is `&'a mut A`.
pub struct StrideMut<'a, A> {
    begin: *mut A,
    offset: int,
    end: int,
    stride: int,
    life: kinds::marker::ContravariantLifetime<'a>,
    nocopy: kinds::marker::NoCopy
}

impl<'a, A> Stride<'a, A>
{
    /// Create a Stride iterator from a raw pointer.
    pub unsafe fn from_ptr_len(begin: *const A, nelem: uint, stride: int) -> Stride<'a, A>
    {
        Stride {
            begin: begin,
            offset: 0,
            end: stride * nelem as int,
            stride: stride,
            life: kinds::marker::ContravariantLifetime,
        }
    }
}

impl<'a, A> StrideMut<'a, A>
{
    /// Create a StrideMut iterator from a raw pointer.
    pub unsafe fn from_ptr_len(begin: *mut A, nelem: uint, stride: int) -> StrideMut<'a, A>
    {
        StrideMut {
            begin: begin,
            offset: 0,
            end: stride * nelem as int,
            stride: stride,
            life: kinds::marker::ContravariantLifetime,
            nocopy: kinds::marker::NoCopy,
        }
    }
}

macro_rules! stride_impl {
    (struct $name:ident -> $slice:ty, $getptr:ident, $ptr:ty, $elem:ty) => {
        impl<'a, A> $name<'a, A>
        {
            /// Create Stride iterator from a slice and the element step count.
            ///
            /// If `step` is negative, start from the back.
            ///
            /// ## Example
            ///
            /// ```
            /// use itertools::Stride;
            ///
            /// let xs = [0i, 1, 2, 3, 4, 5];
            ///
            /// let front = Stride::from_slice(xs.as_slice(), 2);
            /// assert_eq!(front[0], 0);
            /// assert_eq!(front[1], 2);
            ///
            /// let back = Stride::from_slice(xs.as_slice(), -2);
            /// assert_eq!(back[0], 5);
            /// assert_eq!(back[1], 3);
            /// ```
            ///
            /// **Panics** if values of type `A` are zero-sized. <br>
            /// **Panics** if `step` is 0.
            #[inline]
            pub fn from_slice(xs: $slice, step: int) -> $name<'a, A>
            {
                assert!(mem::size_of::<A>() != 0);
                let ustep = if step < 0 { -step } else { step } as uint;
                let nelem = if ustep <= 1 {
                    xs.len()
                } else {
                    let (d, r) = num::div_rem(xs.len(), ustep);
                    d + if r > 0 { 1 } else { 0 }
                };
                let mut begin = xs. $getptr ();
                unsafe {
                    if step > 0 {
                        $name::from_ptr_len(begin, nelem, step)
                    } else {
                        if nelem != 0 {
                            begin = begin.offset(xs.len() as int - 1)
                        }
                        $name::from_ptr_len(begin, nelem, step)
                    }
                }
            }

            /// Create Stride iterator from an existing Stride iterator
            ///
            /// **Panics** if `step` is 0.
            #[inline]
            pub fn from_stride(mut it: $name<'a, A>, mut step: int) -> $name<'a, A>
            {
                assert!(step != 0);
                if step < 0 {
                    it.swap_ends();
                    step = -step;
                }
                let len = (it.end - it.offset) / it.stride;
                let newstride = it.stride * step;
                let (d, r) = num::div_rem(len as uint, step as uint);
                let len = d as uint + if r > 0 { 1 } else { 0 };
                unsafe {
                    $name::from_ptr_len(it.begin, len, newstride)
                }
            }

            /// Swap the begin and end and reverse the stride,
            /// in effect reversing the iterator.
            #[inline]
            pub fn swap_ends(&mut self) {
                let len = (self.end - self.offset) / self.stride;
                if len > 0 {
                    unsafe {
                        let endptr = self.begin.offset((len - 1) * self.stride);
                        *self = $name::from_ptr_len(endptr, len as uint, -self.stride);
                    }
                }
            }

            /// Return the number of elements in the iterator.
            #[inline]
            pub fn len(&self) -> uint {
                ((self.end - self.offset) / self.stride) as uint
            }
        }

        impl<'a, A> Iterator for $name<'a, A>
        {
            type Item = $elem;
            #[inline]
            fn next(&mut self) -> Option<$elem>
            {
                if self.offset == self.end {
                    None
                } else {
                    unsafe {
                        let elt: $elem =
                            mem::transmute(self.begin.offset(self.offset));
                        self.offset += self.stride;
                        Some(elt)
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (uint, Option<uint>) {
                let len = self.len();
                (len, Some(len))
            }
        }

        impl<'a, A> DoubleEndedIterator for $name<'a, A>
        {
            #[inline]
            fn next_back(&mut self) -> Option<$elem>
            {
                if self.offset == self.end {
                    None
                } else {
                    unsafe {
                        self.end -= self.stride;
                        let elt = mem::transmute(self.begin.offset(self.end));
                        Some(elt)
                    }
                }
            }
        }

        impl<'a, A> ExactSizeIterator for $name<'a, A> { }

        impl<'a, A> Index<uint> for $name<'a, A>
        {
            type Output = A;
            /// Return a reference to the element at a given index.
            ///
            /// **Panics** if the index is out of bounds.
            fn index<'b>(&'b self, i: &uint) -> &'b A
            {
                assert!(*i < self.len());
                unsafe {
                    let ptr = self.begin.offset(self.offset + self.stride * (*i as int));
                    mem::transmute(ptr)
                }
            }
        }

        impl<'a, A: fmt::Show> fmt::Show for $name<'a, A>
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
            {
                try!(write!(f, "["));
                for i in range(0, self.len()) {
                    if i != 0 {
                        try!(write!(f, ", "));
                    }
                    try!(write!(f, "{}", (*self)[i]));
                }
                write!(f, "]")
            }
        }
    }
}

stride_impl!{struct Stride -> &'a [A], as_ptr, *const A, &'a A}
stride_impl!{struct StrideMut -> &'a mut [A], as_mut_ptr, *mut A, &'a mut A}

impl<'a, A> Clone for Stride<'a, A>
{
    fn clone(&self) -> Stride<'a, A>
    {
        *self
    }
}

impl<'a, A> IndexMut<uint> for StrideMut<'a, A>
{
    type Output = A;
    /// Return a mutable reference to the element at a given index.
    ///
    /// **Panics** if the index is out of bounds.
    fn index_mut<'b>(&'b mut self, i: &uint) -> &'b mut A
    {
        assert!(*i < self.len());
        unsafe {
            let ptr = self.begin.offset(self.offset + self.stride * (*i as int));
            mem::transmute(ptr)
        }
    }
}
