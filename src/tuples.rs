//! Some iterator that produces tuples

use std::marker::PhantomData;
use super::cloned;

/// An iterator that groups the items in tuples of a specific size.
///
/// See [`.tuples()`](../trait.Itertools.html#method.tuples) for more information.
pub struct Tuples<I, T>
    where I: Iterator
{
    iter: I,
    buf: Option<Vec<I::Item>>,
    _marker: PhantomData<T>,
}

/// Create a new tuples iterator.
pub fn tuples<I, T>(iter: I) -> Tuples<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    Tuples {
        iter: iter,
        buf: None,
        _marker: PhantomData,
    }
}

impl<I, T> Iterator for Tuples<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match T::try_collect_from_iter(&mut self.iter) {
            Ok(v) => Some(v),
            Err(buf) => {
                if self.buf.is_none() {
                    self.buf = Some(buf);
                }
                None
            }
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
        self.buf.unwrap_or_else(|| vec![])
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
    buf: Vec<I::Item>,
    _marker: PhantomData<T>,
}

/// Create a new tuple windows iterator.
pub fn tuple_windows<I, T>(iter: I) -> TupleWindows<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    TupleWindows {
        iter: iter,
        buf: Vec::with_capacity(T::num_items()),
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
        for item in self.iter.by_ref().take(remmaining) {
            self.buf.push(item);
        }
        if self.buf.len() == T::num_items() {
            let r = Some(T::collect_from_iter(cloned(&self.buf)));
            self.buf.remove(0);
            r
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
    /// assert_eq!(vec![4, 5], buffer);
    /// // The next unproduced item is 6
    /// assert_eq!(Some(6), iter.next());
    /// ```
    pub fn into_parts(self) -> (Vec<I::Item>, I) {
        (self.buf, self.iter)
    }
}

pub trait TupleCollect<Item>: Sized {
    fn try_collect_from_iter<I>(iter: I) -> Result<Self, Vec<Item>>
        where I: IntoIterator<Item = Item>;

    fn collect_from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = Item>;

    fn num_items() -> usize;
}

macro_rules! impl_tuple_collect {
    () => ();
    ($A:ident ; $($X:ident),* ; $($Y:ident),*) => (
        impl<$A> TupleCollect<$A> for ($($X),*,) {
            fn try_collect_from_iter<I>(iter: I) -> Result<Self, Vec<$A>>
                where I: IntoIterator<Item = $A>
            {
                let mut iter = iter.into_iter();
                $(let $Y = iter.next();)*

                if $($Y.is_some())&&* {
                    Ok(($($Y.unwrap()),*,))
                } else {
                    let mut v = vec![];
                    $(if let Some(x) = $Y {
                        v.push(x);
                    })*
                    Err(v)
                }
            }

            fn collect_from_iter<I>(iter: I) -> Self
                where I: IntoIterator<Item = $A>
            {
                let mut iter = iter.into_iter();
                // Y must be used, so use it as var name
                ($({let $Y = iter.next().unwrap(); $Y }),*,)
            }

            fn num_items() -> usize {
                // Y must be used, so use it as var name
                0 $(+ { let $Y = 1; $Y})*
            }
        }
    )
}

impl_tuple_collect!(A; A; a);
impl_tuple_collect!(A; A, A; a, b);
impl_tuple_collect!(A; A, A, A; a, b, c);
impl_tuple_collect!(A; A, A, A, A; a, b, c, d);
