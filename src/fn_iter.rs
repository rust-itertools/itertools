/// An iterator that yields values returned by a function.
pub struct FnIter<F>(F);

/// Iterate over a function.
///
/// The returned Iterator calls the passed function on each call to next.
///
/// Example:
///
/// ```
/// // Generate the fibonacci sequence (no recursion).
/// let fib = itertools::func({
///     let mut a = 0;
///     let mut b = 1;
///     move || {
///         let ret = a;
///         let next = a + b;
///         a = b;
///         b = next;
///         Some(ret)
///     }
/// });
///
/// itertools::assert_equal(fib.take(10), vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
/// ```
///
/// 
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
