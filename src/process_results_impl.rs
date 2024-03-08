#[cfg(doc)]
use crate::Itertools;
use core::ptr::NonNull;

/// An iterator that produces only the `T` values as long as the
/// inner iterator produces `Ok(T)`.
///
/// Used by [`process_results`](crate::process_results), see its docs
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct ProcessResults<I, E> {
    error: NonNull<Result<(), E>>,
    iter: I,
}

impl<I, T, E> Iterator for ProcessResults<I, E>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => {
                //SAFETY: the pointer is always valid while iterating over items
                unsafe {
                    *self.error.as_mut() = Err(e);
                }
                None
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }

    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        //SAFETY: the pointer is always valid while iterating over items
        let error = unsafe { self.error.as_mut() };

        self.iter
            .try_fold(init, |acc, opt| match opt {
                Ok(x) => Ok(f(acc, x)),
                Err(e) => {
                    *error = Err(e);
                    Err(acc)
                }
            })
            .unwrap_or_else(|e| e)
    }
}

/// “Lift” a function of the values of an iterator so that it can process
/// an iterator of `Result` values instead.
///
/// [`IntoIterator`] enabled version of [`Itertools::process_results`].
pub fn process_results<I, F, T, E, R>(iterable: I, processor: F) -> Result<R, E>
where
    I: IntoIterator<Item = Result<T, E>>,
    F: FnOnce(ProcessResults<I::IntoIter, E>) -> R,
{
    let iter = iterable.into_iter();
    let mut err = Ok(());

    //SAFETY: the pointer to err will always be valid thoughout the fns lifetime
    let error = unsafe { NonNull::new_unchecked(&mut err) };

    let result = processor(ProcessResults { error, iter });

    err.map(|_| result)
}
