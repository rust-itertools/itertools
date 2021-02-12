
/// An iterator that produces only the `T` values as long as the
/// inner iterator produces `Ok(T)`.
///
/// Used by [`process_results`](../fn.process_results.html), see its docs
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct ProcessResults<'a, I, E: 'a> {
    error: &'a mut Result<(), E>,
    iter: I,
}

impl<'a, I, T, E> Iterator for ProcessResults<'a, I, E>
    where I: Iterator<Item = Result<T, E>>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => {
                *self.error = Err(e);
                None
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

/// “Lift” a function of the values of an iterator so that it can process
/// an iterator of `Result` values instead.
///
/// `iterable` is an iterator or iterable with `Result<T, E>` elements, where
/// `T` is the value type and `E` the error type.
///
/// `processor` is a closure that receives an adapted version of the iterable
/// as the only argument — the adapted iterator produces elements of type `T`,
/// as long as the original iterator produces `Ok` values.
///
/// If the original iterable produces an error at any point, the adapted
/// iterator ends and the `process_results` function will return the
/// error itself.
///
/// Otherwise, the return value from the closure is returned wrapped
/// inside `Ok`.
///
/// See also [`process_results_ref`](./fn.process_results_ref.html).
///
/// # Example
///
/// ```
/// use itertools::process_results;
///
/// type R = Result<i32, &'static str>;
///
/// let first_values: Vec<R> = vec![Ok(1), Ok(0), Ok(3)];
/// let second_values: Vec<R> = vec![Ok(2), Ok(1), Err("overflow")];
///
/// // “Lift” the iterator .max() method to work on the values in Results using process_results
///
/// let first_max = process_results(first_values, |iter| iter.max().unwrap_or(0));
/// let second_max = process_results(second_values, |iter| iter.max().unwrap_or(0));
///
/// assert_eq!(first_max, Ok(3));
/// assert!(second_max.is_err());
/// ```
pub fn process_results<I, F, T, E, R>(iterable: I, processor: F) -> Result<R, E>
    where I: IntoIterator<Item = Result<T, E>>,
          F: FnOnce(ProcessResults<I::IntoIter, E>) -> R
{
    let iter = iterable.into_iter();
    let mut error = Ok(());

    let result = processor(ProcessResults { error: &mut error, iter });

    error.map(|_| result)
}

/// An iterator that produces only the `&T` values as long as the
/// inner iterator produces `&Ok(T)`.
///
/// Used by [`process_results_ref`](../fn.process_results_ref.html), see its docs
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct ProcessResultsRef<'a, 'b, I, E: 'a> {
    error: &'b mut Result<(), &'a E>,
    iter: I,
}

impl<'a, 'b, I, T, E> Iterator for ProcessResultsRef<'a, 'b, I, E>
    where T: 'a,
          E: 'a,
          I: Iterator<Item = &'a Result<T, E>>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => {
                *self.error = Err(e);
                None
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}


/// “Lift” a function of the values of an iterator so that it can process
/// an iterator of `&Result` values instead.
///
/// `iterable` is an iterator or iterable with `&Result<T, E>` elements, where
/// `T` is the value type and `E` the error type.
///
/// `processor` is a closure that receives an adapted version of the iterable
/// as the only argument — the adapted iterator produces elements of type `&T`,
/// as long as the original iterator produces `Ok` values.
///
/// If the original iterable produces an error at any point, the adapted
/// iterator ends and the `process_results_ref` function will return the
/// error itself.
///
/// Otherwise, the return value from the closure is returned wrapped
/// inside `Ok`.
///
/// See also [`process_results`](./fn.process_results.html).
///
/// # Example
///
/// ```
/// use itertools::process_results_ref;
///
/// type R = Result<i32, &'static str>;
///
/// let first_values: Vec<R> = vec![Ok(1), Ok(0), Ok(3)];
/// let second_values: Vec<R> = vec![Ok(2), Ok(1), Err("overflow")];
///
/// // “Lift” the iterator .max() method to work on the values in Results using process_results
///
/// let first_max = process_results_ref(first_values.iter(), |iter| iter.max().unwrap_or(&0));
/// let second_max = process_results_ref(second_values.iter(), |iter| iter.max().unwrap_or(&0));
///
/// assert_eq!(first_max, Ok(&3));
/// assert!(second_max.is_err());
/// ```
pub fn process_results_ref<'a, I, F, T, E, R>(iterable: I, processor: F) -> Result<R, &'a E>
    where T: 'a,
          E: 'a,
          I: IntoIterator<Item = &'a Result<T, E>>,
          F: FnOnce(ProcessResultsRef<'a, '_, I::IntoIter, E>) -> R
{
    let iter = iterable.into_iter();
    let mut error = Ok(());

    let result = processor(ProcessResultsRef { error: &mut error, iter });

    error.map(|_| result)
}
