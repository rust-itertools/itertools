/// An iterator that yields values returned by a function.
pub struct FnIter<F>(F);

/// Iterate over a function.
///
/// The returned Iterator calls the passed function on each call to next.
pub fn func<T, F>(f: F) -> FnIter<F>
    where F: FnMut() -> Option<T>
{
    FnIter(f)
}

impl<T, F> Iterator for FnIter<F>
    where F: FnMut() -> Option<T>
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        (self.0)()
    }
}
