use std::collections::VecDeque;
use std::marker::PhantomData;
use super::chain;

/// An iterator that groups the items in tuples of a specific size.
///
/// See [`.tuples()`](../trait.Itertools.html#method.tuples) for more information.
pub struct Tuples<I, T>
    where I: Iterator
{
    iter: I,
    buf: Vec<I::Item>,
    _marker: PhantomData<T>,
}

/// Create a new tuples iterator.
pub fn tuples<I, T>(iter: I) -> Tuples<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    Tuples {
        iter: iter,
        buf: Vec::with_capacity(T::num_items()),
        _marker: PhantomData,
    }
}

impl<I, T> Iterator for Tuples<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        for (_, item) in (0..T::num_items()).zip(self.iter.by_ref()) {
            self.buf.push(item);
        }
        if self.buf.len() == T::num_items() {
            Some(T::collect_from_iter(self.buf.drain(..)))
        } else {
            None
        }
    }
}

impl<I, T> Tuples<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    /// Return a buffer with the produced items that was not enough to be grouped in a tuple.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut iter = (0..5).tuples();
    /// assert_eq!(Some((0, 1, 2)), iter.next());
    /// assert_eq!(None, iter.next());
    /// assert_eq!(vec![3, 4], iter.into_buffer());
    /// ```
    pub fn into_buffer(self) -> Vec<I::Item> {
        self.buf
    }
}


/// An iterator over all contiguous windows that produces tuples of a specific size.
///
/// See [`.tuple_windows()`](../trait.Itertools.html#method.tuple_windows) for more
/// information.
pub struct TupleWindows<I, T>
    where I: Iterator
{
    iter: I,
    buf: VecDeque<I::Item>,
    _marker: PhantomData<T>,
}

/// Create a new tuple windows iterator.
pub fn tuple_windows<I, T>(iter: I) -> TupleWindows<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    TupleWindows {
        iter: iter,
        buf: VecDeque::with_capacity(T::num_items()),
        _marker: PhantomData,
    }
}

impl<I, T> Iterator for TupleWindows<I, T>
    where I: Iterator,
          I::Item: Clone,
          T: TupleCollect<I::Item>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let remmaining = T::num_items() - self.buf.len();
        for (_, item) in (0..remmaining).zip(self.iter.by_ref()) {
            self.buf.push_back(item);
        }
        if self.buf.len() == T::num_items() {
            let first = self.buf.pop_front();
            let rest = self.buf.iter().cloned();
            Some(T::collect_from_iter(chain(first, rest)))
        } else {
            None
        }
    }
}

impl<I, T> TupleWindows<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    /// Return a pair with the buffer with the already produced items and the inner iterator.
    ///
    /// ```
    /// use itertools::Itertools;
    /// use std::collections::VecDeque;
    ///
    /// let iter = 0..10;
    /// let mut w = iter.tuple_windows();
    /// for (a, _, _) in &mut w {
    ///     if a == 3 {
    ///         break
    ///     }
    /// }
    ///
    /// let (buffer, mut iter) = w.into_parts();
    /// // Items 4 and 5 was already produced
    /// assert_eq!(VecDeque::from(vec![4, 5]), buffer);
    /// // The next unproduced item is 6
    /// assert_eq!(Some(6), iter.next());
    /// ```
    pub fn into_parts(self) -> (VecDeque<I::Item>, I) {
        (self.buf, self.iter)
    }
}

pub trait TupleCollect<Item>: Sized {
    fn collect_from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = Item>;

    fn num_items() -> usize;
}

macro_rules! impl_tuple_collect {
    () => ();
    ($A:ident $($X:ident)*) => (
        impl<$A> TupleCollect<$A> for ($A, $($X),*) {
            #[allow(non_snake_case)]
            fn collect_from_iter<I>(iter: I) -> Self
                where I: IntoIterator<Item = $A>
            {
                let mut iter = iter.into_iter();
                (
                    iter.next().unwrap(),
                    // X must be used, so use it as var name
                    $({let $X = iter.next().unwrap(); $X }),*
                )
            }

            #[allow(non_snake_case)]
            fn num_items() -> usize {
                // X must be used, so use it as var name
                1 $(+ { let $X = 1; $X})*
            }
        }

        impl_tuple_collect!($($X)*);
    )
}

impl_tuple_collect!(A A A A);
