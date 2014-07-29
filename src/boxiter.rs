//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

/// A wrapper around `Box<Iterator<A>>` so that it implements the
/// Iterator trait.
///
/// Iterator element type is `A`
pub struct BoxIter<A> {
    /// The wrapped iterator pointer
    pub iter: Box<Iterator<A>>
}

impl<A> BoxIter<A>
{
    /// Create a BoxIter from an iterator value
    pub fn from_iter<I: 'static + Iterator<A>>(iter: I) -> BoxIter<A>
    {
        BoxIter::from_box(box iter as Box<Iterator<A>>)
    }

    /// Create a BoxIter from an already boxed iterator
    pub fn from_box(iter: Box<Iterator<A>>) -> BoxIter<A>
    {
        BoxIter{iter: iter}
    }
}

impl<A> Iterator<A> for BoxIter<A>
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
