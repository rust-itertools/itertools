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
                    self.inner = None;
                }
            } else {
                match self.iter.next() {
                    Some(Ok(ok)) => self.inner = Some(ok.into_iter()),
                    Some(Err(e)) => return Some(Err(e)),
                    None => return None,
                }
            }
        }
    }
}

impl<I, T, E> Clone for FlattenOk<I, T, E>
where
    I: Iterator<Item = Result<T, E>> + Clone,
    T: IntoIterator,
    T::IntoIter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            inner: self.inner.clone(),
        }
    }
}
