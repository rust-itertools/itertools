use std::iter::Peekable;
use crate::PutBack;
#[cfg(feature = "use_std")]
use crate::PutBackN;

/// A trait for folding an iterator by peeking at its elements before
/// consuming them.
pub trait PeekingFoldWhile : Iterator {
    /// An iterator method that applies a function to each element
    /// as long as it returns successfully, producing a single value.
    ///
    /// See [`.peeking_fold_while()`](../trait.Itertools.html#method.peeking_fold_while) for
    /// more information.
    fn peeking_fold_while<T, E, F>(&mut self, init: T, f: F) -> Result<T, E>
        where F: FnMut(T, &Self::Item) -> Result<T, E>;
}

impl<I> PeekingFoldWhile for Peekable<I>
    where I:Iterator
{
    fn peeking_fold_while<T, E, F>(&mut self, init: T, mut f: F) -> Result<T, E>
        where F: FnMut(T, &I::Item) -> Result<T, E>,
    {
        let mut acc = init;
        while let Some(x) = self.peek() {
            let result = f(acc, x);
            if result.is_ok() {
                self.next();
            }
            acc = result?;
        }
        Ok(acc)
    }
}

impl<I> PeekingFoldWhile for PutBack<I>
    where I: Iterator
{
    fn peeking_fold_while<T, E, F>(&mut self, init: T, mut f: F) -> Result<T, E>
        where F: FnMut(T, &I::Item) -> Result<T, E>,
    {
        let mut acc = init;
        while let Some(x) = self.next() {
            let result = f(acc, &x);
            if result.is_err() {
                self.put_back(x);
            }
            acc = result?;
        }
        Ok(acc)
    }
}

#[cfg(feature = "use_std")]
impl<I> PeekingFoldWhile for PutBackN<I>
    where I: Iterator
{
    fn peeking_fold_while<T, E, F>(&mut self, init: T, mut f: F) -> Result<T, E>
        where F: FnMut(T, &I::Item) -> Result<T, E>,
    {
        let mut acc = init;
        while let Some(x) = self.next() {
            let result = f(acc, &x);
            if result.is_err() {
                self.put_back(x);
            }
            acc = result?;
        }
        Ok(acc)
    }
}
