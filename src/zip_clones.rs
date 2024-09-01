/// An iterator which iterate over an iterator and clones of a value.
///
///
/// See [`.zip_clones()`](crate::Itertools::zip_clones) for more information.
pub struct ZipClones<I, T>
where
    I: Iterator,
{
    iter: I,
    next: Option<I::Item>,
    zipped: Option<T>,
}

/// Zips an iterator with clones of a value.
///
/// [`IntoIterator`] enabled version of [`Itertools::zip_clones`](crate::Itertools::zip_clones).
///
/// ```
/// use itertools::zip_clones;
///
/// let data = [1, 2, 3, 4, 5];
/// let zipped = "expensive-to-clone".to_string();
/// for (a, b) in zip_clones(&data, zipped) {
///     // do something that consumes the expensive zipped value
/// }
/// ```
pub fn zip_clones<I, T>(i: I, zipped: T) -> ZipClones<I::IntoIter, T>
where
    I: IntoIterator,
    T: Clone,
{
    let mut iter = i.into_iter();
    let next = iter.next();
    ZipClones {
        iter,
        next,
        zipped: Some(zipped),
    }
}

impl<I: Iterator, T: Clone> Iterator for ZipClones<I, T> {
    type Item = (I::Item, T);
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.next.take()?;
        self.next = self.iter.next();
        let zipped = if self.next.is_some() {
            self.zipped.clone()
        } else {
            self.zipped.take()
        };
        // Safety: the zipped field is only self.next is none
        let zipped = unsafe { zipped.unwrap_unchecked() };
        Some((cur, zipped))
    }
}

#[cfg(test)]
mod tests {
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
