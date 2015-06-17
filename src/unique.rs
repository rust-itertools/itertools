//! Adds the method *.unique()* to **Iterator**.

use std::collections::HashSet;
use std::hash::Hash;

/// Struct for storing which elements we've already seen.
pub struct UniqueState<I>
    where I: Iterator
{
    seen: HashSet<I::Item>,
    underlying: I,
}

/// An iterator that discards duplicates.
///
/// ```
/// use itertools::Unique;
///
/// let before = vec!["a", "b", "c", "b"];
/// let after: Vec<_> = before.into_iter().unique().collect();
///
/// assert_eq!(after, vec!["a", "b", "c"]);
/// ```
pub trait Unique: Iterator {
    /// Create a new **Unique**.
    fn unique(self) -> UniqueState<Self>
        where Self::Item: Hash + Eq + Clone,
              Self: Sized,
    {
        UniqueState { seen: HashSet::new(), underlying: self }
    }
}

impl<I> Unique for I where I: Iterator {}

impl<I> Iterator for UniqueState<I>
    where I: Iterator,
          I::Item: Hash + Eq + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.underlying.next() {
            if !self.seen.contains(&x) {
                self.seen.insert(x.clone());
                return Some(x)
            }
        }
        None
    }
}
