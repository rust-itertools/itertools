//! Some iterator that produces tuples

use std::marker::PhantomData;
use std::iter::Fuse;

/// An iterator that groups the items in tuples of a specific size.
///
/// See [`.tuples()`](../trait.Itertools.html#method.tuples) for more information.
pub struct Tuples<I, T>
    where I: Iterator
{
    iter: Fuse<I>,
    buf: Option<Vec<I::Item>>,
    _marker: PhantomData<T>,
}

/// Create a new tuples iterator.
pub fn tuples<I, T>(iter: I) -> Tuples<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    Tuples {
        iter: iter.fuse(),
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
    iter: Fuse<I>,
    last: Option<T>,
    buf: Option<Vec<I::Item>>,
    _marker: PhantomData<T>,
}

/// Create a new tuple windows iterator.
pub fn tuple_windows<I, T>(iter: I) -> TupleWindows<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item>
{
    TupleWindows {
        iter: iter.fuse(),
        last: None,
        buf: None,
        _marker: PhantomData,
    }
}

impl<I, T> Iterator for TupleWindows<I, T>
    where I: Iterator,
          I::Item: Clone,
          T: TupleCollect<I::Item> + Clone
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if let Some(ref mut last) = self.last {
            if let Some(new) = self.iter.next() {
                last.left_shift_push(new);
                Some(last.clone())
            } else {
                self.buf = Some(vec![]);
                None
            }
        } else {
            match T::try_collect_from_iter(&mut self.iter) {
                Ok(v) => {
                    self.last = Some(v);
                    self.last.clone()
                },
                Err(buf) => {
                    if self.buf.is_none() {
                        self.buf = Some(buf);
                    }
                    None
                }
            }
        }
    }
}

impl<I, T> TupleWindows<I, T>
    where I: Iterator,
          T: TupleCollect<I::Item> + ::std::fmt::Debug
{
    /// Return a pair with a buffer containing the items that was produced but not consumed and the
    /// inner iterator.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut w = (0..10).tuple_windows();
    /// assert_eq!(Some((0, 1, 2)), w.next());
    ///
    /// let (buffer, mut iter) = w.into_parts();
    /// // Every produced item was consumed
    /// assert!(buffer.is_empty());
    /// // The next item is 6
    /// assert_eq!(Some(3), iter.next());
    ///
    /// let mut w = (0..2).tuple_windows::<(_, _, _)>();
    /// assert_eq!(None, w.next());
    ///
    /// let (buffer, mut iter) = w.into_parts();
    /// // The items 0 and 1 was produced but not consumed
    /// assert_eq!(vec![0, 1], buffer);
    /// // The is no more items
    /// assert_eq!(None, iter.next());
    /// ```
    pub fn into_parts(self) -> (Vec<I::Item>, Fuse<I>) {
        (self.buf.unwrap_or_else(|| vec![]), self.iter)
    }
}

pub trait TupleCollect<Item>: Sized {
    fn try_collect_from_iter<I>(iter: I) -> Result<Self, Vec<Item>>
        where I: IntoIterator<Item = Item>;

    fn collect_from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = Item>;

    fn left_shift_push(&mut self, item: Item);

    fn into_vec(self) -> Vec<Item>;
}

macro_rules! impl_tuple_collect {
    () => ();
    ($A:ident ; $($X:ident),* ; $($Y:ident),* ; $($Y_rev:ident),*) => (
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

            fn left_shift_push(&mut self, item: $A) {
                use std::mem::replace;

                let &mut ($(ref mut $Y),*,) = self;
                let tmp = item;
                $(
                    let tmp = replace($Y_rev, tmp);
                )*
                drop(tmp);
            }

            fn into_vec(self) -> Vec<$A> {
                let ($($Y),*,) = self;
                let mut v = vec![];
                $(
                    v.push($Y);
                )*
                v
            }
        }
    )
}

impl_tuple_collect!(A; A; a; a);
impl_tuple_collect!(A; A, A; a, b; b, a);
impl_tuple_collect!(A; A, A, A; a, b, c; c, b, a);
impl_tuple_collect!(A; A, A, A, A; a, b, c, d; d, c, b, a);
