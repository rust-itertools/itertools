/// Iterator returned for the error case of `IterTools::exactly_one()`
/// This iterator yields exactly the same elements as the input iterator.
///
/// During the execution of exactly_one the iterator must be mutated.  This wrapper
/// effectively "restores" the state of the input iterator when it's handed back.
///
/// This is very similar to PutBackN except this iterator only supports 0-2 elements and does not
/// use a `Vec`.
#[derive(Debug, Clone)]
pub struct ExactlyOneErr<T, I>
where
    I: Iterator<Item = T>,
{
    first_two: (Option<T>, Option<T>),
    inner: I,
}

impl<T, I> ExactlyOneErr<T, I>
where
    I: Iterator<Item = T>,
{
    /// Creates a new `ExactlyOneErr` iterator.
    pub fn new(first_two: (Option<T>, Option<T>), inner: I) -> Self {
        Self { first_two, inner }
    }
}

impl<T, I> Iterator for ExactlyOneErr<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.first_two
            .0
            .take()
            .or_else(|| self.first_two.1.take())
            .or_else(|| self.inner.next())
    }
}
