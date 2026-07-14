use std::{iter::FusedIterator, time::{Duration, Instant}};


/// Tracks and returns durations of each call to `next()`.
/// 
/// See [`crate::Itertools::timed()`] for more details.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct TimedIterator<I> {
    iter: I,
    exhausted: bool,
}

impl<I: Iterator> TimedIterator<I> {
    /// Creates a new TimedIterator that times the given input
    /// iterator's calls to `next()` when iterated over.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            exhausted: false,
        }
    }
}

impl<I: Iterator> Iterator for TimedIterator<I> {
    type Item = (Option<I::Item>, Duration);

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            None
        } else {
            let start = Instant::now();
            let item = self.iter.next();
            let duration = start.elapsed();
            self.exhausted = item.is_none();
            Some((item, duration))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.exhausted {
            (0, Some(0))
        } else {
            let (lower_limit, upper_limit) = self.iter.size_hint();
            let lower_limit = match lower_limit {
                usize::MAX => usize::MAX,
                lower_limit => lower_limit + 1,
            };
            let upper_limit = match upper_limit {
                None => None,
                Some(usize::MAX) => None,
                Some(upper_limit) => Some(upper_limit + 1),
            };
            (lower_limit, upper_limit)
        }
    }
}

impl<I: Iterator> FusedIterator for TimedIterator<I> {}
