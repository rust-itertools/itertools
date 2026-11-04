use std::iter::Peekable;

/// An iterator which iterate over an iterator and clones of a value.
///
///
/// See [`.zip_clones()`](crate::Itertools::zip_clones) for more information.
pub struct ZipClones<I, T>
where
    I: Iterator,
{
    iter: Peekable<I>,
    cloned: Option<T>,
}

/// Zips an iterator with clones of a value.
///
/// [`IntoIterator`] enabled version of [`Itertools::zip_clones`](crate::Itertools::zip_clones).
///
/// ```
/// use itertools::Itertools;
///
/// let data = [1, 2, 3, 4, 5];
/// let zipped = "expensive-to-clone".to_string();
/// for (a, b) in data.iter().zip_clones(zipped) {
///     // do something that consumes the expensive zipped value
///     drop((a, b));
/// }
/// ```
pub fn zip_clones<I, T>(i: I, zipped: T) -> ZipClones<I::IntoIter, T>
where
    I: IntoIterator,
    T: Clone,
{
    ZipClones {
        iter: i.into_iter().peekable(),
        cloned: Some(zipped),
    }
}

impl<I: Iterator, T: Clone> Iterator for ZipClones<I, T> {
    type Item = (I::Item, T);
    fn next(&mut self) -> Option<Self::Item> {
        // let cur = self.next.take()?;
        let cur = self.iter.next()?;
        let zipped = if self.iter.peek().is_some() {
            self.cloned.clone()
        } else {
            self.cloned.take()
        };
        // Safety: the zipped field is Some as long as the iterator is not exhausted
        let zipped = unsafe { zipped.unwrap_unchecked() };
        Some((cur, zipped))
    }
}

#[cfg(test)]
mod tests {
    use crate::Itertools;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_zip_clones() {
        static ZIPPED_CLONES_COUNTER: AtomicUsize = AtomicUsize::new(0);
        struct Zipped {}
        impl Clone for Zipped {
            fn clone(&self) -> Self {
                ZIPPED_CLONES_COUNTER.fetch_add(1, Ordering::SeqCst);
                Zipped {}
            }
        }

        ZIPPED_CLONES_COUNTER.store(0, Ordering::SeqCst);
        let iter_len = [1, 2, 3, 4, 5, 6].iter().zip_clones(Zipped {}).count();
        assert_eq!(iter_len, 6);
        assert_eq!(ZIPPED_CLONES_COUNTER.load(Ordering::SeqCst), 5);
    }
}
