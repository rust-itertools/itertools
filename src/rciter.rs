
use std::iter::IntoIterator;
use std::rc::Rc;
use std::cell::RefCell;

/// A wrapper for `Rc<RefCell<I>>`, that implements the `Iterator` trait.
///
/// See [*.into_rc()*](trait.Itertools.html#method.into_rc) for more information.
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

impl<A, I> Iterator for RcIter<I> where
    I: Iterator<Item=A>,
{
    type Item = A;
    #[inline]
    fn next(&mut self) -> Option<A>
    {
        self.rciter.borrow_mut().next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // To work sanely with other API that assume they own an iterator,
        // so it can't change in other places, we can't guarantee as much
        // in our size_hint. Other clones may drain values under our feet.
        let (_, hi) = self.rciter.borrow().size_hint();
        (0, hi)
    }
}

impl<I> DoubleEndedIterator for RcIter<I> where
    I: DoubleEndedIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<I::Item>
    {
        self.rciter.borrow_mut().next_back()
    }
}
/// Return an iterator from `&RcIter<I>` (by simply cloning it).
impl<'a, I> IntoIterator for &'a RcIter<I> where
    I: Iterator,
{
    type Item = I::Item;
    type IntoIter = RcIter<I>;

    fn into_iter(self) -> RcIter<I>
    {
        self.clone()
    }
}
