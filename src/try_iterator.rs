mod private {
    pub trait Sealed {}
    impl<I, T, E> Sealed for I where I: Iterator<Item = Result<T, E>> {}
}

/// Helper trait automatically implemented for [`Iterator`]s of [`Result`]s.
///
/// Can be useful for specifying certain trait bounds more concisely. Take
/// [`.err_into()`][err_into] for example:
///
/// Without [`TryIterator`], [`err_into`][err_into] would have to be generic
/// over 3 type parameters: the type of [`Result::Ok`] values, the type of
/// [`Result::Err`] values, and the type to convert errors into. Usage would
/// look like this: `my_iterator.err_into<_, _, E>()`.
///
/// Using [`TryIterator`], [`err_into`][err_into] can be generic over a single
/// type parameter, and called like this: `my_iterator.err_into<E>()`.
///
/// [err_into]: crate::Itertools::err_into
pub trait TryIterator: Iterator + private::Sealed {
    /// The type of [`Result::Ok`] values yielded by this [`Iterator`].
    type Ok;

    /// The type of [`Result::Err`] values yielded by this [`Iterator`].
    type Error;
}

impl<I, T, E> TryIterator for I
where
    I: Iterator<Item = Result<T, E>>,
{
    type Ok = T;
    type Error = E;
}
