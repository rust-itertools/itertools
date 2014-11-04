//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

/// A wrapper around `Box<Iterator<A>>` so that it implements the
/// Iterator trait.
///
/// Iterator element type is `A`
pub struct BoxIter<I> {
    /// The wrapped iterator pointer
    pub iter: Box<I>
}

impl<A, I> BoxIter<I>
    where I: Iterator<A>
{
    /// Create a BoxIter from an iterator value
    pub fn from_iter(iter: I) -> BoxIter<I>
    {
        BoxIter::from_box(box iter)
    }

    /// Create a BoxIter from an already boxed iterator
    pub fn from_box(iter: Box<I>) -> BoxIter<I>
    {
        BoxIter{iter: iter}
    }
}

impl<A, I> Iterator<A> for BoxIter<I>
    where I: Iterator<A>
{
    #[inline]
    fn next(&mut self) -> Option<A>
    {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>)
    {
        self.iter.size_hint()
    }
}
