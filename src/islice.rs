use super::Itertools;
use super::misc::GenericRange;

/// A sliced iterator.
///
/// **Note:** slicing an iterator is not constant time, and much less efficient than
/// slicing for example a vector.
///
/// ## Example
/// ```
/// # extern crate itertools;
/// # fn main() {
/// use std::iter::repeat;
/// # use itertools::Itertools;
///
/// let mut it = repeat('a').slice(..3);
/// assert_eq!(it.count(), 3);
/// # }
/// ```
#[derive(Copy, Clone)]
pub struct ISlice<I> {
    start: usize,
    end: usize,
    iter: I,
}

impl<I> ISlice<I>
    where I: Iterator
{
    /// Create a new **ISlice**.
    pub fn new<R: GenericRange>(iter: I, range: R) -> Self
    {
        ISlice {
            start: range.start().unwrap_or(0),
            end: range.end().unwrap_or(::std::usize::MAX),
            iter: iter,
        }
    }
}

impl<I> Iterator for ISlice<I>
    where I: Iterator
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item>
    {
        if self.start != 0 {
            let st = self.start;
            let n = self.iter.dropn(self.start);
            self.start = 0;
            self.end -= n;
            if n != st {
                // iterator is already done.
                return None
            }
        }
        if self.end != 0 {
            self.end -= 1;
            self.iter.next()
        } else {
            None
        }
    }
}
