/// An iterator adaptor that combines each element except the first with a clone of the previous.
///
/// See [`.with_prev()`](crate::Itertools::with_prev) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct WithPrev<I>
where
    I: Iterator,
{
    iter: I,
    prev: Option<I::Item>,
}

impl<I> Clone for WithPrev<I>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    clone_fields!(iter, prev);
}

/// Create a new `WithPrev` iterator.
pub fn with_prev<I>(iter: I) -> WithPrev<I>
where
    I: Iterator,
{
    WithPrev { iter, prev: None }
}

impl<I> Iterator for WithPrev<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (Option<I::Item>, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        let prev = std::mem::replace(&mut self.prev, Some(next.clone()));
        Some((prev, next))
    }
}
