
use std::rc::Rc;
use std::cell::RefCell;

/// A wrapper for `Rc<RefCell<I>>`, that implements the `Iterator` trait.
pub struct RcIter<I> {
    /// The boxed iterator.
    pub rciter: Rc<RefCell<I>>,
}

impl<I> RcIter<I>
{
    /// Create a new RcIter.
    pub fn new(iter: I) -> RcIter<I>
    {
        RcIter{rciter: Rc::new(RefCell::new(iter))}
    }
}
impl<I> Clone for RcIter<I>
{
    #[inline]
    fn clone(&self) -> RcIter<I> {
        RcIter{rciter: self.rciter.clone()}
    }
}

impl<A, I: Iterator<Item=A>> Iterator for RcIter<I>
{
    type Item = A;
    #[inline]
    fn next(&mut self) -> Option<A>
    {
        self.rciter.borrow_mut().next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.rciter.borrow().size_hint()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for RcIter<I>
{
    #[inline]
    fn next_back(&mut self) -> Option< <I as Iterator>::Item>
    {
        self.rciter.borrow_mut().next_back()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for RcIter<I> {}
