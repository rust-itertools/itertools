use std::num::Saturating;

/// An iterator adaptor to insert a particular value
/// between each element of the adapted iterator.
///
/// Iterator element type is `A`
#[deriving(Clone)]
pub struct Intersperse<A, I> {
    element: A,
    iter: I,
    peek: Option<A>,
}

impl<A, I: Iterator<A>> Intersperse<A, I>
{
    /// Create a new Intersperse iterator
    pub fn new(mut iter: I, elt: A) -> Intersperse<A, I>
    {
        Intersperse{peek: iter.next(), iter: iter, element: elt}
    }
}

impl<A: Clone, I: Iterator<A>>
Iterator<A> for Intersperse<A, I>
{
    #[inline]
    fn next(&mut self) -> Option<A>
    {
        if self.peek.is_some() {
            self.peek.take()
        } else {
            self.peek = self.iter.next();
            if self.peek.is_some() {
                Some(self.element.clone())
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>)
    {
        let (mut low, mut hi) = self.iter.size_hint();
        if low > 0 {
            low = low.saturating_add((low - 1));
        }
        hi = hi.and_then(|x| if x > 0 {
            x.checked_add(&(x - 1))
        } else { Some (x) });
        (low, hi)
    }
}
