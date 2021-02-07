use std::{fmt, iter::FusedIterator};

pub fn flatten_ok<I, T, E>(iter: I) -> FlattenOk<I, T, E>
where
    I: Iterator<Item = Result<T, E>>,
    T: IntoIterator,
{
    FlattenOk { iter, inner: None }
}

/// An iterator adaptor that flattens `Result::Ok` values and
/// allows `Result::Err` values through unchanged.
///
/// See [`.flatten_ok()`](crate::Itertools::flatten_ok) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct FlattenOk<I, T, E>
where
    I: Iterator<Item = Result<T, E>>,
    T: IntoIterator,
{
    iter: I,
    inner: Option<T::IntoIter>,
}

impl<I, T, E> Iterator for FlattenOk<I, T, E>
where
    I: Iterator<Item = Result<T, E>>,
    T: IntoIterator,
{
    type Item = Result<T::Item, E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(inner) = &mut self.inner {
                if let Some(item) = inner.next() {
                    return Some(Ok(item));
                } else {
                    // This is necessary for the iterator to implement `FusedIterator`
                    // with only the orginal iterator being fused.
                    self.inner = None;
                }
            }

            match self.iter.next() {
                Some(Ok(ok)) => self.inner = Some(ok.into_iter()),
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(inner) = &self.inner {
            // If we have an inner iterator, then its lower bound is our lower bound,
            // but we still don't know the upper bound.
            let (inner_lower, inner_higher) = inner.size_hint();
            let (_, outer_higher) = self.iter.size_hint();
            if outer_higher == Some(0) {
                // If there is nothing remaining in the outer iterator, we know the upper bound.
                (inner_lower, inner_higher)
            } else {
                // However, if the outer iterator could have more items in it, we don't
                // know the upper bound.
                (inner_lower, None)
            }
        } else if self.iter.size_hint() == (0, Some(0)) {
            // If the outer iterator is empty, we have no items.
            (0, Some(0))
        } else {
            // Otherwise we do not know anything about the number of items.
            (0, None)
        }
    }
}

impl<I, T, E> Clone for FlattenOk<I, T, E>
where
    I: Iterator<Item = Result<T, E>> + Clone,
    T: IntoIterator,
    T::IntoIter: Clone,
{
    #[inline]
    clone_fields!(iter, inner);
}

impl<I, T, E> fmt::Debug for FlattenOk<I, T, E>
where
    I: Iterator<Item = Result<T, E>> + fmt::Debug,
    T: IntoIterator,
    T::IntoIter: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlattenOk")
            .field("iter", &self.iter)
            .field("inner", &self.inner)
            .finish()
    }
}

/// Only the iterator being flattened needs to implement [`FusedIterator`].
impl<I, T, E> FusedIterator for FlattenOk<I, T, E>
where
    I: FusedIterator<Item = Result<T, E>>,
    T: IntoIterator,
{
}
